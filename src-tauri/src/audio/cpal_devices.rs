use super::catalog::{AudioError, RawInventory, RefreshPermit};

#[cfg(any(target_os = "windows", target_os = "macos"))]
mod capture;
#[cfg(any(target_os = "windows", target_os = "macos"))]
mod playback;

// Gate every callback before a native stream destructor can block, including
// input-only failures after play(). Children share this single stream owner.
#[cfg(any(target_os = "windows", target_os = "macos"))]
struct Streams {
    input: Option<cpal::Stream>,
    output: Option<cpal::Stream>,
    run: std::sync::Arc<super::capture::Run>,
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
impl Streams {
    fn new(run: &std::sync::Arc<super::capture::Run>) -> Self {
        Self {
            input: None,
            output: None,
            run: std::sync::Arc::clone(run),
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
impl Drop for Streams {
    fn drop(&mut self) {
        self.run.quiet();
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub(super) fn test(
    run: &std::sync::Arc<super::capture::Run>,
    selection: &super::capture::Selection,
) -> Result<(), AudioError> {
    if selection.mode == super::capture::Mode::Input {
        capture::capture(run, selection.input.as_deref())
    } else {
        playback::play(run, selection)
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub(super) fn test(
    _: &std::sync::Arc<super::capture::Run>,
    _: &super::capture::Selection,
) -> Result<(), AudioError> {
    Err(AudioError::new(
        "unsupported_platform",
        "Native audio tests currently support Linux, Windows and macOS.",
    ))
}

#[cfg(target_os = "windows")]
pub const BACKEND: &str = "WASAPI";
#[cfg(target_os = "macos")]
pub const BACKEND: &str = "CoreAudio";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub const BACKEND: &str = "Unsupported";

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn host() -> Result<cpal::Host, AudioError> {
    #[cfg(target_os = "windows")]
    let id = cpal::HostId::Wasapi;
    #[cfg(target_os = "macos")]
    let id = cpal::HostId::CoreAudio;
    cpal::host_from_id(id).map_err(map_error)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn configuration(
    supported: &cpal::SupportedStreamConfig,
    default_buffer: bool,
) -> Result<cpal::StreamConfig, AudioError> {
    let mut config = supported.config();
    if !(1..=2).contains(&config.channels) || config.sample_rate == 0 {
        return Err(super::capture::unsupported());
    }
    // CoreAudio's Fixed buffer request changes a device-wide property. Keep
    // its current OS buffer; Windows retains the approved ~10 ms / Default retry.
    if default_buffer || cfg!(target_os = "macos") {
        config.buffer_size = cpal::BufferSize::Default;
    } else if let cpal::SupportedBufferSize::Range { min, max } = *supported.buffer_size() {
        if min > max || max == 0 {
            return Err(super::capture::unsupported());
        }
        config.buffer_size = cpal::BufferSize::Fixed((config.sample_rate / 100).clamp(min, max));
    }
    Ok(config)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn map_error(error: cpal::Error) -> AudioError {
    use cpal::ErrorKind;
    match error.kind() {
        ErrorKind::PermissionDenied => AudioError::new(
            "permission_denied",
            "Audio device access was denied. Check the system audio permissions.",
        ),
        ErrorKind::HostUnavailable => AudioError::new(
            "host_unavailable",
            "The system audio service is unavailable. Check it and Refresh.",
        ),
        ErrorKind::DeviceNotAvailable => AudioError::new(
            "device_unavailable",
            "An audio device disappeared. Refresh devices.",
        ),
        ErrorKind::DeviceBusy => AudioError::new(
            "device_busy",
            "An audio device is busy. Close its other user and Refresh.",
        ),
        ErrorKind::UnsupportedConfig | ErrorKind::UnsupportedOperation => {
            super::capture::unsupported()
        }
        _ => AudioError::new(
            "backend_error",
            "Audio devices could not be read. Check the system audio configuration and Refresh.",
        ),
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn enumerate(scan: &RefreshPermit) -> Result<RawInventory, AudioError> {
    use super::catalog::{RawDevice, MAX_DEVICES};
    use cpal::traits::{DeviceTrait, HostTrait};

    scan.check()?;
    let host = host()?;

    let default_name = |device: Option<cpal::Device>| -> Result<Option<String>, AudioError> {
        device
            .map(|d| {
                d.description()
                    .map(|v| v.name().to_owned())
                    .map_err(map_error)
            })
            .transpose()
    };
    let mut result = RawInventory {
        default_input: default_name(host.default_input_device())?,
        default_output: default_name(host.default_output_device())?,
        ..RawInventory::default()
    };
    for (index, device) in host
        .devices()
        .map_err(map_error)?
        .take(2 * MAX_DEVICES + 1)
        .enumerate()
    {
        if index >= 2 * MAX_DEVICES {
            return Err(AudioError::new(
                "inventory_limit",
                "Too many audio devices were reported. Reduce virtual devices and Refresh.",
            ));
        }
        scan.check()?;
        let id = device.id().map_err(map_error)?;
        let name = device.description().map_err(map_error)?.name().to_owned();
        // Both backends check native endpoint capabilities without opening a stream.
        // In particular, output loopback capability does not make a microphone.
        if device.supports_input() {
            result.inputs.push(RawDevice {
                native_id: id.to_string(),
                label: name.clone(),
            });
        }
        if device.supports_output() {
            result.outputs.push(RawDevice {
                native_id: id.to_string(),
                label: name,
            });
        }
        if result.inputs.len() > MAX_DEVICES || result.outputs.len() > MAX_DEVICES {
            return Err(AudioError::new(
                "inventory_limit",
                "Too many audio devices were reported. Reduce virtual devices and Refresh.",
            ));
        }
    }
    Ok(result)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn enumerate(_scan: &RefreshPermit) -> Result<RawInventory, AudioError> {
    Err(AudioError::new(
        "unsupported_platform",
        "Native audio device settings currently support Linux, Windows and macOS.",
    ))
}

#[cfg(all(test, any(target_os = "windows", target_os = "macos")))]
mod tests {
    use super::*;
    use cpal::{BufferSize, SampleFormat, SupportedBufferSize, SupportedStreamConfig};

    #[test]
    fn buffer_policy_keeps_coreaudio_default_and_bounds_wasapi_requests() {
        for (min, max) in [(64, 1024), (64, 128), (512, 1024)] {
            let supported = SupportedStreamConfig::new(
                2,
                48000,
                SupportedBufferSize::Range { min, max },
                SampleFormat::F32,
            );
            let config = configuration(&supported, false).unwrap();
            assert_eq!(config.sample_rate, 48000);
            assert_eq!(config.channels, 2);
            let expected = if cfg!(target_os = "macos") {
                BufferSize::Default
            } else {
                BufferSize::Fixed(480u32.clamp(min, max))
            };
            assert_eq!(config.buffer_size, expected);
            assert_eq!(
                configuration(&supported, true).unwrap().buffer_size,
                BufferSize::Default
            );
        }
    }

    #[test]
    fn invalid_native_layouts_and_rates_cannot_start_a_test() {
        for (channels, rate) in [(0, 48000), (6, 48000), (1, 0)] {
            let supported = SupportedStreamConfig::new(
                channels,
                rate,
                SupportedBufferSize::Unknown,
                SampleFormat::F32,
            );
            assert_eq!(
                configuration(&supported, false).unwrap_err().code,
                "unsupported_config"
            );
        }
    }
}
