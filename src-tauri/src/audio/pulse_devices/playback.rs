//! Playback stays in the Pulse mainloop worker. Native callbacks only set
//! flags; writable buffers are borrowed, initialized, committed or cancelled.
use super::capture::{buffer_limits, on_moved, on_read, Endpoint, InputStream, Stream};
use super::*;
use crate::audio::{
    capture::{self as test, CaptureFormat, Mode, Run, Selection},
    playout::{self, Signal},
};
use std::{ffi::CString, mem::MaybeUninit, sync::Arc};

extern "C" fn on_output(
    _: *mut pulse::pa_context,
    info: *const pulse::pa_sink_info,
    end: i32,
    data: *mut c_void,
) {
    // SAFETY: Query<Endpoint> owns the stable reply; info lives only in callback.
    let Some(reply) = (unsafe { (data as *mut Reply<Endpoint>).as_mut() }) else {
        return;
    };
    if reply.error.is_some() {
        return;
    }
    if end != 0 {
        reply.done = true;
        if end < 0 || reply.value.name.is_empty() {
            reply.error = Some(test::unavailable());
        }
        return;
    }
    reply.error = (|| {
        let sink = unsafe { info.as_ref() }.ok_or_else(test::unavailable)?;
        let name = unsafe { text(sink.name, 1024) }?
            .filter(|s| !s.is_empty())
            .ok_or_else(test::unavailable)?;
        if sink.sample_spec.rate == 0 || !(1..=2).contains(&sink.sample_spec.channels) {
            return Err(test::unsupported());
        }
        reply.value = Endpoint {
            name,
            rate: sink.sample_spec.rate,
            channels: sink.sample_spec.channels,
        };
        Ok(())
    })()
    .err();
}

fn output(
    context: &Context,
    mainloop: &mut EventLoop,
    run: &Arc<Run>,
    selected: Option<&str>,
) -> Result<Endpoint, AudioError> {
    let name = match selected {
        Some(id) => id
            .strip_prefix("pulse:output:")
            .filter(|s| !s.is_empty())
            .ok_or_else(test::unavailable)?
            .to_owned(),
        None => defaults(context, mainloop, run)?
            .output
            .ok_or_else(test::unavailable)?,
    };
    let name = CString::new(name).map_err(|_| test::unavailable())?;
    let result = Query::<Endpoint>::new(context, |data| {
        // SAFETY: correct sink callback/userdata, live Ready context and name.
        unsafe {
            pulse::pa_context_get_sink_info_by_name(
                context.raw.as_ptr(),
                name.as_ptr(),
                Some(on_output),
                data,
            )
        }
    })?
    .wait(mainloop, run)?;
    if result.name.as_bytes() != name.as_bytes() {
        return Err(test::unavailable());
    }
    Ok(result)
}

struct Output<'a> {
    stream: Stream<'a>,
    format: CaptureFormat,
    maximum: usize,
}

fn open_output<'a>(
    context: &'a Context,
    mainloop: &mut EventLoop,
    run: &Arc<Run>,
    info: &Endpoint,
) -> Result<Output<'a>, AudioError> {
    let (maximum, fragment) = buffer_limits(info.rate, info.channels)?;
    let name = CString::new(info.name.as_str()).map_err(|_| test::unavailable())?;
    let spec = pulse::pa_sample_spec {
        format: pulse::PA_SAMPLE_FLOAT32NE,
        rate: info.rate,
        channels: info.channels,
    };
    let stream = Stream::new(context, run, c"Local output test", &spec)?;
    let data = (&*stream.signals as *const capture::Signals)
        .cast_mut()
        .cast();
    let attributes = pulse::pa_buffer_attr {
        maxlength: maximum,
        tlength: fragment,
        prebuf: 0,
        minreq: fragment,
        fragsize: u32::MAX,
    };
    // SAFETY: stable signal storage; callbacks only flag work on this worker.
    let result = unsafe {
        pulse::pa_stream_set_write_callback(stream.raw.as_ptr(), Some(on_read), data);
        pulse::pa_stream_set_moved_callback(stream.raw.as_ptr(), Some(on_moved), data);
        pulse::pa_stream_connect_playback(
            stream.raw.as_ptr(),
            name.as_ptr(),
            &attributes,
            pulse::PA_STREAM_START_CORKED
                | pulse::PA_STREAM_DONT_MOVE
                | pulse::PA_STREAM_ADJUST_LATENCY,
            std::ptr::null(),
            std::ptr::null_mut(),
        )
    };
    if result < 0 {
        return Err(context.error());
    }
    stream.wait_ready(mainloop, run)?;
    stream.validate_endpoint(&info.name, &spec)?;
    // SAFETY: Ready stream lends the negotiated buffer attributes for this check.
    let actual = unsafe { pulse::pa_stream_get_buffer_attr(stream.raw.as_ptr()).as_ref() }
        .ok_or_else(test::unsupported)?;
    if actual.maxlength > maximum
        || actual.tlength > maximum
        || actual.minreq > maximum
        || actual.tlength == 0
    {
        return Err(test::unsupported());
    }
    let format = CaptureFormat {
        sample_rate: spec.rate,
        channels: u16::from(spec.channels),
        sample_format: "f32",
        buffer_frames: Some(actual.tlength / (4 * u32::from(spec.channels))),
    };
    Ok(Output {
        stream,
        format,
        maximum: maximum as usize,
    })
}

struct Reservation<'a> {
    stream: &'a Stream<'a>,
    committed: bool,
}
impl Drop for Reservation<'_> {
    fn drop(&mut self) {
        if !self.committed {
            // SAFETY: constructed only after a successful begin_write; no
            // subsequent write/cancel occurred on this reservation.
            unsafe {
                pulse::pa_stream_cancel_write(self.stream.raw.as_ptr());
            }
        }
    }
}

impl Output<'_> {
    fn pump(&self, run: &Run, signal: &mut Signal, force: bool) -> Result<(), AudioError> {
        self.stream.ready()?;
        if !self.stream.signals.readable.replace(false) && !force {
            return Ok(());
        }
        run.check()?;
        let frame_bytes = usize::from(self.format.channels) * 4;
        // SAFETY: local cached writable count, not a blocking server query.
        let writable = unsafe { pulse::pa_stream_writable_size(self.stream.raw.as_ptr()) };
        if writable == usize::MAX {
            return Err(backend_error());
        }
        let mut bytes = writable.min(self.maximum) / frame_bytes * frame_bytes;
        if bytes == 0 {
            return Ok(());
        }
        let requested = bytes;
        let mut data = std::ptr::null_mut();
        // SAFETY: begin_write lends storage until commit or Reservation::drop.
        if unsafe { pulse::pa_stream_begin_write(self.stream.raw.as_ptr(), &mut data, &mut bytes) }
            != 0
            || data.is_null()
        {
            return Err(backend_error());
        }
        let mut reservation = Reservation {
            stream: &self.stream,
            committed: false,
        };
        if bytes > requested || bytes < frame_bytes {
            return Err(test::unsupported());
        }
        bytes = bytes / frame_bytes * frame_bytes;
        run.stats.output_buffer(bytes / frame_bytes);
        {
            // SAFETY: native storage may be uninitialized, hence MaybeUninit;
            // every byte submitted below is written before the C API reads it.
            let output =
                unsafe { std::slice::from_raw_parts_mut(data.cast::<MaybeUninit<u8>>(), bytes) };
            signal.render(
                bytes / frame_bytes,
                run.stats.now(),
                run.enabled(),
                |index, frame| {
                    for (channel, sample) in frame
                        .iter()
                        .take(usize::from(self.format.channels))
                        .enumerate()
                    {
                        let encoded = sample.to_ne_bytes();
                        let base = index * frame_bytes + channel * 4;
                        for byte in 0..4 {
                            output[base + byte].write(encoded[byte]);
                        }
                    }
                },
            );
        }
        run.check()?; // cancellation also cancels the unwritten reservation
                      // SAFETY: the initialized prefix is frame-aligned and within the lease.
        if unsafe {
            pulse::pa_stream_write(
                self.stream.raw.as_ptr(),
                data,
                bytes,
                None,
                0,
                pulse::PA_SEEK_RELATIVE,
            )
        } < 0
        {
            return Err(backend_error());
        }
        reservation.committed = true;
        // Drain another bounded chunk on the next control iteration if needed.
        if writable > bytes {
            self.stream.signals.readable.set(true);
        }
        Ok(())
    }
}

pub(super) fn play(run: &Arc<Run>, selection: &Selection) -> Result<(), AudioError> {
    with_context(run, None, |context, mainloop| {
        let sink = output(context, mainloop, run, selection.output.as_deref())?;
        let source = if run.mode == Mode::Loopback {
            Some(capture::input(
                context,
                mainloop,
                run,
                selection.input.as_deref(),
            )?)
        } else {
            None
        };
        if source
            .as_ref()
            .is_some_and(|source| source.rate != sink.rate)
        {
            return Err(playout::incompatible());
        }
        let mut input: Option<InputStream<'_>> = None;
        let mut producer = None;
        let mut signal = if let Some(source) = source {
            let (tx, rx) =
                playout::queue(sink.rate, u16::from(sink.channels), Arc::clone(&run.stats))?;
            producer = Some(tx);
            input = Some(capture::open_input(context, mainloop, run, &source, true)?);
            Signal::Loopback(rx)
        } else {
            Signal::Tone {
                rate: sink.rate,
                frame: 0,
            }
        };
        let output = open_output(context, mainloop, run, &sink)?;
        if let Some(input) = &input {
            input.stream.uncork(mainloop, run)?;
            input.drain(run, producer.as_mut())?;
        }
        output.pump(run, &mut signal, true)?;
        output.stream.uncork(mainloop, run)?;
        run.started_playback(
            input.as_ref().map(|input| input.format.clone()),
            output.format.clone(),
        )?;
        loop {
            run.check()?;
            context.check()?;
            if let Some(input) = &input {
                input.drain(run, producer.as_mut())?;
            }
            output.pump(run, &mut signal, false)?;
            step(mainloop, run)?;
        }
    })
}
