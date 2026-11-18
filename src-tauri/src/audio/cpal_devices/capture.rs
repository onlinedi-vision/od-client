//! WASAPI/CoreAudio: resolve Default to a specific endpoint; never follow it.
use super::{configuration, host, map_error, Streams};
use crate::audio::{
    budget::Budget,
    capture::{self, CaptureFormat, Run},
    catalog::{AudioError, MAX_DEVICES},
};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, SampleFormat,
};
use std::{sync::Arc, time::Duration};

pub(in crate::audio) fn capture(run: &Arc<Run>, selected: Option<&str>) -> Result<(), AudioError> {
    run.check()?;
    let host = host()?;
    let id = match selected {
        Some(id) => id.to_owned(),
        None => host
            .default_input_device()
            .ok_or_else(capture::unavailable)?
            .id()
            .map_err(map_error)?
            .to_string(),
    };
    let mut chosen = None;
    for device in host.devices().map_err(map_error)?.take(2 * MAX_DEVICES) {
        run.check()?;
        if device.id().map_err(map_error)?.to_string() == id && device.supports_input() {
            chosen = Some(device);
            break;
        }
    }
    let device = chosen.ok_or_else(capture::unavailable)?;
    let fallback = device.default_input_config().map_err(map_error)?;
    let mut preferred = None;
    // CoreAudio can change device-wide physical formats during build. Request
    // only its current format/rate, rather than preferring a new 48 kHz format.
    if cfg!(target_os = "windows") {
        for range in device
            .supported_input_configs()
            .map_err(map_error)?
            .take(128)
        {
            run.check()?;
            if (1..=2).contains(&range.channels()) && range.sample_format() == SampleFormat::F32 {
                if let Some(config) = range.try_with_sample_rate(48000) {
                    preferred = Some(config);
                    break;
                }
            }
        }
    }
    let supported = preferred.unwrap_or(fallback);
    let mut config = configuration(&supported, false)?;
    let make = |config| -> Result<cpal::Stream, cpal::Error> {
        match supported.sample_format() {
            SampleFormat::F32 => build::<f32>(&device, config, run, None),
            SampleFormat::I16 => build::<i16>(&device, config, run, None),
            SampleFormat::U16 => build::<u16>(&device, config, run, None),
            SampleFormat::I32 => build::<i32>(&device, config, run, None),
            _ => Err(cpal::Error::new(cpal::ErrorKind::UnsupportedConfig)),
        }
    };
    run.check()?;
    let stream = match make(config) {
        Err(error)
            if error.kind() == cpal::ErrorKind::UnsupportedConfig
                && matches!(config.buffer_size, BufferSize::Fixed(_)) =>
        {
            // One approved buffer-size fallback on this same endpoint only.
            run.check()?;
            config.buffer_size = BufferSize::Default;
            make(config).map_err(map_error)?
        }
        result => result.map_err(map_error)?,
    };
    let mut streams = Streams::new(run);
    let stream = streams.input.insert(stream);
    run.check()?; // delayed build after Stop must drop without play
    stream.play().map_err(map_error)?;
    run.started(CaptureFormat {
        sample_rate: config.sample_rate,
        channels: config.channels,
        sample_format: match supported.sample_format() {
            SampleFormat::F32 => "f32",
            SampleFormat::I16 => "i16",
            SampleFormat::U16 => "u16",
            SampleFormat::I32 => "i32",
            _ => return Err(capture::unsupported()),
        },
        buffer_frames: match config.buffer_size {
            BufferSize::Fixed(frames) => Some(frames),
            _ => None,
        },
    })?;
    loop {
        let remaining = run.check()?;
        std::thread::sleep(remaining.min(Duration::from_millis(25)));
    }
    // Streams gates callbacks before dropping its input handle on every exit.
}

pub(super) fn build<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    run: &Arc<Run>,
    mut producer: Option<crate::audio::playout::Producer>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    use cpal::Sample;
    // Callbacks retain only the shared meter/cancel gate, not the controller.
    let samples = Arc::clone(run);
    let errors = Arc::clone(&samples);
    device.build_input_stream_raw(
        config,
        T::FORMAT,
        move |data, info| {
            if !samples.enabled() {
                return;
            }
            if let Some(buffer) = data.as_slice::<T>() {
                let decoded = buffer.iter().map(|value| value.to_sample::<f32>());
                samples.meter.samples(decoded.clone());
                if let Some(producer) = &mut producer {
                    let stamp = info.timestamp();
                    if let Some(age) = stamp.callback.checked_duration_since(stamp.capture) {
                        producer.push(decoded, config.channels, samples.stats.captured_at(age));
                    } else {
                        samples.stats.timing_miss();
                        samples.meter.fail(4);
                    }
                }
            } else {
                samples.meter.fail(1);
            }
        },
        move |error| {
            match error.kind() {
                cpal::ErrorKind::Xrun => errors.meter.overrun(),
                cpal::ErrorKind::PermissionDenied => errors.meter.fail(2),
                cpal::ErrorKind::DeviceNotAvailable
                | cpal::ErrorKind::DeviceChanged
                | cpal::ErrorKind::StreamInvalidated => errors.meter.fail(3),
                cpal::ErrorKind::RealtimeDenied => {} // valid stream; scheduling promotion was refused
                _ => errors.meter.fail(1),
            }
        },
        Some(Duration::from_secs(5)),
    )
}
