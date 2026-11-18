//! WASAPI/CoreAudio output/duplex. One queue connects the two native callbacks;
//! all capability queries, builds, retries and drops run on the control worker.
use super::{capture as input, configuration, host, map_error, Streams};
use crate::audio::{
    budget::Budget,
    capture::{self as test, CaptureFormat, Mode, Run, Selection},
    catalog::{AudioError, MAX_DEVICES},
    formats,
    playout::{self, Producer, Signal},
};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, SampleFormat, SupportedStreamConfig, SupportedStreamConfigRange,
};
use std::{sync::Arc, time::Duration};

struct Endpoint {
    device: cpal::Device,
    configs: Vec<SupportedStreamConfigRange>,
}
fn endpoint(
    host: &cpal::Host,
    run: &Run,
    selected: Option<&str>,
    capture: bool,
) -> Result<Endpoint, AudioError> {
    run.check()?;
    let id = match selected {
        Some(id) => id.to_owned(),
        None => (if capture {
            host.default_input_device()
        } else {
            host.default_output_device()
        })
        .ok_or_else(test::unavailable)?
        .id()
        .map_err(map_error)?
        .to_string(),
    };
    for device in host.devices().map_err(map_error)?.take(2 * MAX_DEVICES) {
        run.check()?;
        if device.id().map_err(map_error)?.to_string() != id
            || !(if capture {
                device.supports_input()
            } else {
                device.supports_output()
            })
        {
            continue;
        }
        #[cfg(target_os = "windows")]
        let configs = if capture {
            device
                .supported_input_configs()
                .map_err(map_error)?
                .take(128)
                .collect()
        } else {
            device
                .supported_output_configs()
                .map_err(map_error)?
                .take(128)
                .collect()
        };
        #[cfg(target_os = "macos")]
        let configs = {
            // Advertise only the current OS format to the common selector.
            // Different active rates require an explicit OS change. CPAL's own
            // physical-format setup still needs hardware validation.
            let current = if capture {
                device.default_input_config()
            } else {
                device.default_output_config()
            }
            .map_err(map_error)?;
            vec![SupportedStreamConfigRange::new(
                current.channels(),
                current.sample_rate(),
                current.sample_rate(),
                *current.buffer_size(),
                current.sample_format(),
            )]
        };
        return Ok(Endpoint { device, configs });
    }
    Err(test::unavailable())
}

fn ranges(endpoint: &Endpoint) -> Vec<formats::Range> {
    endpoint
        .configs
        .iter()
        .map(|config| formats::Range {
            min: if matches!(
                config.sample_format(),
                SampleFormat::F32 | SampleFormat::I16 | SampleFormat::U16 | SampleFormat::I32
            ) {
                config.min_sample_rate()
            } else {
                0
            },
            max: config.max_sample_rate(),
            channels: config.channels(),
            float: config.sample_format() == SampleFormat::F32,
        })
        .collect()
}

fn description(
    config: cpal::StreamConfig,
    kind: SampleFormat,
) -> Result<CaptureFormat, AudioError> {
    Ok(CaptureFormat {
        sample_rate: config.sample_rate,
        channels: config.channels,
        sample_format: match kind {
            SampleFormat::F32 => "f32",
            SampleFormat::I16 => "i16",
            SampleFormat::U16 => "u16",
            SampleFormat::I32 => "i32",
            _ => return Err(test::unsupported()),
        },
        buffer_frames: if let BufferSize::Fixed(frames) = config.buffer_size {
            Some(frames)
        } else {
            None
        },
    })
}

fn input_stream(
    endpoint: &Endpoint,
    config: cpal::StreamConfig,
    kind: SampleFormat,
    run: &Arc<Run>,
    producer: Producer,
) -> Result<cpal::Stream, AudioError> {
    match kind {
        SampleFormat::F32 => input::build::<f32>(&endpoint.device, config, run, Some(producer)),
        SampleFormat::I16 => input::build::<i16>(&endpoint.device, config, run, Some(producer)),
        SampleFormat::U16 => input::build::<u16>(&endpoint.device, config, run, Some(producer)),
        SampleFormat::I32 => input::build::<i32>(&endpoint.device, config, run, Some(producer)),
        _ => return Err(test::unsupported()),
    }
    .map_err(map_error)
}

fn output_stream(
    endpoint: &Endpoint,
    config: cpal::StreamConfig,
    kind: SampleFormat,
    run: &Arc<Run>,
    signal: Signal,
) -> Result<cpal::Stream, AudioError> {
    match kind {
        SampleFormat::F32 => build_output::<f32>(&endpoint.device, config, run, signal),
        SampleFormat::I16 => build_output::<i16>(&endpoint.device, config, run, signal),
        SampleFormat::U16 => build_output::<u16>(&endpoint.device, config, run, signal),
        SampleFormat::I32 => build_output::<i32>(&endpoint.device, config, run, signal),
        _ => return Err(test::unsupported()),
    }
    .map_err(map_error)
}

fn build_output<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    run: &Arc<Run>,
    mut signal: Signal,
) -> Result<cpal::Stream, cpal::Error>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    use cpal::Sample;
    let output = Arc::clone(run);
    let errors = Arc::clone(run);
    device.build_output_stream_raw(
        config,
        T::FORMAT,
        move |data, _| {
            if let Some(buffer) = data.as_slice_mut::<T>() {
                let channels = usize::from(config.channels);
                if buffer.len() % channels != 0 {
                    buffer.fill(T::from_sample(0.0));
                    output.meter.fail(1);
                    return;
                }
                output.stats.output_buffer(buffer.len() / channels);
                signal.render(
                    buffer.len() / channels,
                    output.stats.now(),
                    output.enabled(),
                    |index, frame| {
                        for (channel, sample) in frame.iter().take(channels).enumerate() {
                            buffer[index * channels + channel] = T::from_sample(*sample);
                        }
                    },
                );
            } else {
                output.meter.fail(1);
            }
        },
        move |error| match error.kind() {
            cpal::ErrorKind::Xrun => errors.meter.overrun(),
            cpal::ErrorKind::PermissionDenied => errors.meter.fail(2),
            cpal::ErrorKind::DeviceNotAvailable
            | cpal::ErrorKind::DeviceChanged
            | cpal::ErrorKind::StreamInvalidated => errors.meter.fail(3),
            cpal::ErrorKind::RealtimeDenied => {}
            _ => errors.meter.fail(1),
        },
        Some(Duration::from_secs(5)),
    )
}

fn attempt(
    run: &Arc<Run>,
    source: Option<(&Endpoint, &SupportedStreamConfig)>,
    sink: &Endpoint,
    output: &SupportedStreamConfig,
    default_buffer: bool,
) -> Result<(), AudioError> {
    let out_config = configuration(output, default_buffer)?;
    let mut streams = Streams::new(run);
    let mut input_format = None;
    let signal = if let Some((source, supported)) = source {
        let in_config = configuration(supported, default_buffer)?;
        let (tx, rx) = playout::queue(
            out_config.sample_rate,
            out_config.channels,
            Arc::clone(&run.stats),
        )?;
        input_format = Some(description(in_config, supported.sample_format())?);
        streams.input = Some(input_stream(
            source,
            in_config,
            supported.sample_format(),
            run,
            tx,
        )?);
        Signal::Loopback(rx)
    } else {
        Signal::Tone {
            rate: out_config.sample_rate,
            frame: 0,
        }
    };
    streams.output = Some(output_stream(
        sink,
        out_config,
        output.sample_format(),
        run,
        signal,
    )?);
    run.check()?;
    if let Some(input) = &streams.input {
        input.play().map_err(map_error)?;
    }
    run.check()?;
    let output_stream = streams.output.as_ref().ok_or_else(test::unsupported)?;
    output_stream.play().map_err(map_error)?;
    run.started_playback(
        input_format,
        description(out_config, output.sample_format())?,
    )?;
    loop {
        let remaining = run.check()?;
        std::thread::sleep(remaining.min(Duration::from_millis(25)));
    }
}

pub(super) fn play(run: &Arc<Run>, selection: &Selection) -> Result<(), AudioError> {
    run.check()?;
    let host = host()?;
    let sink = endpoint(&host, run, selection.output.as_deref(), false)?;
    let source = if run.mode == Mode::Loopback {
        Some(endpoint(&host, run, selection.input.as_deref(), true)?)
    } else {
        None
    };
    let out_ranges = ranges(&sink);
    let in_ranges = source.as_ref().map(ranges);
    let (i, o, rate) = formats::common(in_ranges.as_deref().unwrap_or(&out_ranges), &out_ranges)
        .ok_or_else(playout::incompatible)?;
    let out_config = sink.configs[o]
        .try_with_sample_rate(rate)
        .ok_or_else(playout::incompatible)?;
    let in_config = source
        .as_ref()
        .map(|source| {
            source.configs[i]
                .try_with_sample_rate(rate)
                .ok_or_else(playout::incompatible)
        })
        .transpose()?;
    let input = source.as_ref().zip(in_config.as_ref());
    match attempt(run, input, &sink, &out_config, false) {
        Err(error) if cfg!(target_os = "windows") && error.code == "unsupported_config" => {
            // Previous handles/callbacks have dropped before this one retry.
            // Same endpoints, formats/rate; only driver buffer defaults change.
            run.retry_setup()?;
            attempt(run, input, &sink, &out_config, true)
        }
        result => result,
    }
}
