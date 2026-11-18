//! Explicit microphone capture. Read callbacks set flags only; bounded PCM
//! inspection happens on the owning worker, never through JSON or on disk.
use super::*;
use crate::audio::capture::{self, CaptureFormat, Run};
use std::ffi::CString;
use std::sync::Arc;

#[derive(Default)]
pub(super) struct Endpoint {
    pub name: String,
    pub rate: u32,
    pub channels: u8,
}

extern "C" fn on_input(
    _: *mut pulse::pa_context,
    info: *const pulse::pa_source_info,
    end: i32,
    data: *mut c_void,
) {
    // SAFETY: Query<Endpoint> owns this stable Box until cancel/unref; libpulse
    // lends info only for this invocation. No pointer escapes the callback.
    let Some(reply) = (unsafe { (data as *mut Reply<Endpoint>).as_mut() }) else {
        return;
    };
    if reply.error.is_some() {
        return;
    }
    if end != 0 {
        reply.done = true;
        if end < 0 || reply.value.name.is_empty() {
            reply.error = Some(capture::unavailable());
        }
        return;
    }
    reply.error = (|| {
        let source = unsafe { info.as_ref() }.ok_or_else(capture::unavailable)?;
        if source.monitor_of_sink != pulse::PA_INVALID_INDEX {
            return Err(capture::unavailable());
        }
        let name = unsafe { text(source.name, 1024) }?
            .filter(|s| !s.is_empty())
            .ok_or_else(capture::unavailable)?;
        if !(1..=2).contains(&source.sample_spec.channels) || source.sample_spec.rate == 0 {
            return Err(capture::unsupported());
        }
        reply.value = Endpoint {
            name,
            rate: source.sample_spec.rate,
            channels: source.sample_spec.channels,
        };
        Ok(())
    })()
    .err();
}

#[derive(Default)]
pub(super) struct Signals {
    pub readable: Cell<bool>,
    pub moved: Cell<bool>,
}

pub(super) extern "C" fn on_read(_: *mut pulse::pa_stream, _: usize, data: *mut c_void) {
    // SAFETY: Stream owns the registered Signals until it unregisters callbacks.
    if let Some(signals) = unsafe { (data as *const Signals).as_ref() } {
        signals.readable.set(true);
    }
}
pub(super) extern "C" fn on_moved(_: *mut pulse::pa_stream, data: *mut c_void) {
    // SAFETY: same owner and worker as on_read.
    if let Some(signals) = unsafe { (data as *const Signals).as_ref() } {
        signals.moved.set(true);
    }
}
extern "C" fn on_uncork(_: *mut pulse::pa_stream, success: i32, data: *mut c_void) {
    on_subscribed(std::ptr::null_mut(), success, data);
}

pub(super) struct Stream<'a> {
    pub raw: NonNull<pulse::pa_stream>,
    pub signals: Box<Signals>,
    run: Arc<Run>,
    _context: &'a Context,
}

impl Drop for Stream<'_> {
    fn drop(&mut self) {
        self.run.quiet();
        // SAFETY: outside dispatch, all operations dropped; callbacks disabled
        // before freeing userdata, stream before context/mainloop.
        unsafe {
            pulse::pa_stream_set_read_callback(self.raw.as_ptr(), None, std::ptr::null_mut());
            pulse::pa_stream_set_moved_callback(self.raw.as_ptr(), None, std::ptr::null_mut());
            pulse::pa_stream_set_write_callback(self.raw.as_ptr(), None, std::ptr::null_mut());
            pulse::pa_stream_disconnect(self.raw.as_ptr());
            pulse::pa_stream_unref(self.raw.as_ptr());
        }
    }
}

pub(super) fn buffer_limits(rate: u32, channels: u8) -> Result<(u32, u32), AudioError> {
    if rate == 0 || !(1..=2).contains(&channels) {
        return Err(capture::unsupported());
    }
    let frame = 4 * u32::from(channels);
    let maximum = ((u64::from(rate) * 40 / 1000).min(u64::from(65536 / frame)) as u32) * frame;
    let fragment = (rate / 100)
        .checked_mul(frame)
        .filter(|n| *n > 0 && *n <= maximum)
        .ok_or_else(capture::unsupported)?;
    Ok((maximum, fragment))
}

impl<'a> Stream<'a> {
    pub fn new(
        context: &'a Context,
        run: &Arc<Run>,
        name: &CStr,
        spec: &pulse::pa_sample_spec,
    ) -> Result<Self, AudioError> {
        run.check()?;
        // SAFETY: live context, fixed valid name and validated sample specification.
        let raw = unsafe {
            pulse::pa_stream_new(context.raw.as_ptr(), name.as_ptr(), spec, std::ptr::null())
        };
        Ok(Self {
            raw: NonNull::new(raw).ok_or_else(|| context.error())?,
            signals: Box::default(),
            run: Arc::clone(run),
            _context: context,
        })
    }
    pub fn ready(&self) -> Result<(), AudioError> {
        // SAFETY: this object owns the stream reference throughout.
        if unsafe { pulse::pa_stream_get_state(self.raw.as_ptr()) } != pulse::PA_STREAM_READY
            || self.signals.moved.get()
        {
            return Err(capture::unavailable());
        }
        Ok(())
    }
    pub fn validate_endpoint(
        &self,
        name: &str,
        spec: &pulse::pa_sample_spec,
    ) -> Result<(), AudioError> {
        // SAFETY: the caller has waited for Ready; this stream owns the borrowed
        // negotiated metadata. Neither pointer escapes this check.
        let actual = unsafe { pulse::pa_stream_get_sample_spec(self.raw.as_ptr()).as_ref() }
            .ok_or_else(capture::unsupported)?;
        if actual.format != spec.format
            || actual.channels != spec.channels
            || actual.rate != spec.rate
        {
            return Err(capture::unsupported());
        }
        let actual_name =
            unsafe { text(pulse::pa_stream_get_device_name(self.raw.as_ptr()), 1024) }?;
        if actual_name.as_deref() != Some(name) {
            return Err(capture::unavailable());
        }
        Ok(())
    }
    pub fn wait_ready(&self, mainloop: &mut EventLoop, run: &Arc<Run>) -> Result<(), AudioError> {
        loop {
            run.check()?;
            // SAFETY: owned live stream on the context's worker.
            match unsafe { pulse::pa_stream_get_state(self.raw.as_ptr()) } {
                pulse::PA_STREAM_READY => return self.ready(),
                pulse::PA_STREAM_FAILED | pulse::PA_STREAM_TERMINATED => {
                    return Err(self._context.error())
                }
                _ => step(mainloop, run)?,
            }
        }
    }
    pub fn uncork(&self, mainloop: &mut EventLoop, run: &Arc<Run>) -> Result<(), AudioError> {
        run.check()?;
        Query::<()>::new(self._context, |data| {
            // SAFETY: operation/stream/context live through the owning query.
            unsafe { pulse::pa_stream_cork(self.raw.as_ptr(), 0, Some(on_uncork), data) }
        })?
        .wait(mainloop, run)
    }
}

pub(super) fn input(
    context: &Context,
    mainloop: &mut EventLoop,
    run: &Arc<Run>,
    selected: Option<&str>,
) -> Result<Endpoint, AudioError> {
    let name = match selected {
        Some(id) => id
            .strip_prefix("pulse:input:")
            .filter(|s| !s.is_empty())
            .ok_or_else(capture::unavailable)?
            .to_owned(),
        None => defaults(context, mainloop, run)?
            .input
            .ok_or_else(capture::unavailable)?,
    };
    let name = CString::new(name).map_err(|_| capture::unavailable())?;
    let input = Query::<Endpoint>::new(context, |data| {
        // SAFETY: live Ready context, valid name, matching owned callback reply.
        unsafe {
            pulse::pa_context_get_source_info_by_name(
                context.raw.as_ptr(),
                name.as_ptr(),
                Some(on_input),
                data,
            )
        }
    })?
    .wait(mainloop, run)?;
    if input.name.as_bytes() != name.as_bytes() {
        return Err(capture::unavailable());
    }
    Ok(input)
}

pub(super) struct InputStream<'a> {
    pub stream: Stream<'a>,
    pub format: CaptureFormat,
    maximum: u32,
}

impl InputStream<'_> {
    pub fn drain(
        &self,
        run: &Run,
        producer: Option<&mut crate::audio::playout::Producer>,
    ) -> Result<(), AudioError> {
        self.stream.ready()?;
        if self.stream.signals.readable.replace(false) {
            drain(
                &self.stream,
                run,
                self.maximum as usize,
                usize::from(self.format.channels),
                producer,
            )?;
        }
        Ok(())
    }
}

pub(super) fn open_input<'a>(
    context: &'a Context,
    mainloop: &mut EventLoop,
    run: &Arc<Run>,
    input: &Endpoint,
    timed: bool,
) -> Result<InputStream<'a>, AudioError> {
    let name = CString::new(input.name.as_str()).map_err(|_| capture::unavailable())?;
    let (maximum, fragment) = buffer_limits(input.rate, input.channels)?;
    let spec = pulse::pa_sample_spec {
        format: pulse::PA_SAMPLE_FLOAT32NE,
        rate: input.rate,
        channels: input.channels,
    };
    let stream = Stream::new(context, run, c"Microphone test", &spec)?;
    let data = (&*stream.signals as *const Signals).cast_mut().cast();
    let attributes = pulse::pa_buffer_attr {
        maxlength: maximum,
        tlength: u32::MAX,
        prebuf: u32::MAX,
        minreq: u32::MAX,
        fragsize: fragment,
    };
    let timing = if timed {
        pulse::PA_STREAM_AUTO_TIMING_UPDATE | pulse::PA_STREAM_INTERPOLATE_TIMING
    } else {
        pulse::PA_STREAM_NOFLAGS
    };
    // SAFETY: stable userdata and owned C string; no I/O in the flag callbacks.
    let result = unsafe {
        pulse::pa_stream_set_read_callback(stream.raw.as_ptr(), Some(on_read), data);
        pulse::pa_stream_set_moved_callback(stream.raw.as_ptr(), Some(on_moved), data);
        pulse::pa_stream_connect_record(
            stream.raw.as_ptr(),
            name.as_ptr(),
            &attributes,
            pulse::PA_STREAM_START_CORKED
                | pulse::PA_STREAM_DONT_MOVE
                | pulse::PA_STREAM_ADJUST_LATENCY
                | timing,
        )
    };
    if result < 0 {
        return Err(context.error());
    }
    stream.wait_ready(mainloop, run)?;
    stream.validate_endpoint(&input.name, &spec)?;
    // SAFETY: Ready stream lends the negotiated buffer attributes for this check.
    let buffer = unsafe { pulse::pa_stream_get_buffer_attr(stream.raw.as_ptr()).as_ref() }
        .ok_or_else(capture::unsupported)?;
    if buffer.maxlength > maximum || buffer.fragsize > maximum || buffer.fragsize == 0 {
        return Err(capture::unsupported());
    }
    let format = CaptureFormat {
        sample_rate: spec.rate,
        channels: u16::from(spec.channels),
        sample_format: "f32",
        buffer_frames: Some(buffer.fragsize / (4 * u32::from(spec.channels))),
    };
    if timed {
        Query::<()>::new(context, |data| {
            // SAFETY: Ready stream and owned reply; initialize capture timing.
            unsafe {
                pulse::pa_stream_update_timing_info(stream.raw.as_ptr(), Some(on_uncork), data)
            }
        })?
        .wait(mainloop, run)?;
    }
    Ok(InputStream {
        stream,
        format,
        maximum,
    })
}

pub(in crate::audio) fn capture(run: &Arc<Run>, selected: Option<&str>) -> Result<(), AudioError> {
    with_context(run, None, |context, mainloop| {
        let info = input(context, mainloop, run, selected)?;
        let input = open_input(context, mainloop, run, &info, false)?;
        input.stream.uncork(mainloop, run)?;
        run.started(input.format.clone())?;
        loop {
            run.check()?;
            context.check()?;
            input.drain(run, None)?;
            step(mainloop, run)?;
        }
    })
}

// as_chunks would require Rust 1.88 solely for this Clippy style suggestion.
#[allow(clippy::chunks_exact_to_as_chunks)]
fn drain(
    stream: &Stream<'_>,
    run: &Run,
    maximum: usize,
    channels: usize,
    mut producer: Option<&mut crate::audio::playout::Producer>,
) -> Result<(), AudioError> {
    // Bound both fragment count and bytes; discard overflow rather than queueing
    // old audio. No application PCM buffer is allocated or retained.
    let mut inspected = 0usize;
    for _ in 0..16 {
        run.check()?;
        let mut ptr = std::ptr::null();
        let mut bytes = 0;
        // SAFETY: peek lends a fragment until the matching drop below.
        if unsafe { pulse::pa_stream_peek(stream.raw.as_ptr(), &mut ptr, &mut bytes) } < 0 {
            return Err(backend_error());
        }
        if bytes == 0 {
            return Ok(());
        }
        if bytes > maximum || bytes % (4 * channels) != 0 {
            // SAFETY: release the lent fragment even on rejected metadata.
            unsafe {
                pulse::pa_stream_drop(stream.raw.as_ptr());
            }
            return Err(capture::unsupported());
        }
        inspected = inspected.saturating_add(bytes);
        if ptr.is_null() || inspected > maximum {
            run.meter.silence();
            run.meter.overrun();
        } else {
            // SAFETY: libpulse lends exactly bytes readable bytes until drop;
            // byte decoding avoids any alignment assumption or reinterpret cast.
            let samples = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), bytes) };
            let decoded = samples
                .chunks_exact(4)
                .map(|s| f32::from_ne_bytes([s[0], s[1], s[2], s[3]]));
            run.meter.samples(decoded.clone());
            if let Some(producer) = producer.as_deref_mut() {
                let mut usec = 0;
                let mut negative = 0;
                // SAFETY: local cached/interpolated capture timing, no round trip.
                if unsafe {
                    pulse::pa_stream_get_latency(stream.raw.as_ptr(), &mut usec, &mut negative)
                } == 0
                    && negative == 0
                {
                    producer.push(
                        decoded,
                        channels as u16,
                        run.stats
                            .captured_at(std::time::Duration::from_micros(usec)),
                    );
                } else {
                    run.stats.timing_miss();
                    // No fabricated fresh timestamp: release the peek below,
                    // then the worker reports the structured timing failure.
                    run.meter.fail(4);
                }
            }
        }
        // SAFETY: every nonempty peek is matched once, after borrowed data expires.
        if unsafe { pulse::pa_stream_drop(stream.raw.as_ptr()) } < 0 {
            return Err(backend_error());
        }
    }
    // More fragments: return to control checks; never extend work indefinitely.
    stream.signals.readable.set(true);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_buffers_are_checked_bounded_and_frame_aligned() {
        for rate in [44100, 48000, 96000, 768000] {
            for channels in [1, 2] {
                let (maximum, fragment) = buffer_limits(rate, channels).unwrap();
                assert!(maximum <= 65536 && maximum <= rate * 40 / 1000 * 4 * u32::from(channels));
                assert!(fragment > 0 && fragment <= maximum);
                assert_eq!(maximum % (4 * u32::from(channels)), 0);
            }
        }
        assert!(buffer_limits(0, 1).is_err());
        assert!(buffer_limits(48000, 0).is_err());
        assert!(buffer_limits(48000, 3).is_err());
    }
}
