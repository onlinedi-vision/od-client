//! Policy for desktop endpoints; no libpulse handles, server indexes or PCM.
use super::{backend_error, changed_error};
use crate::audio::catalog::{AudioError, RawDevice, RawInventory, MAX_DEVICES};

#[derive(Default, PartialEq, Eq)]
pub(super) struct Defaults {
    pub input: Option<String>,
    pub output: Option<String>,
}

#[derive(Default)]
pub(super) struct Endpoints {
    pub seen: usize,
    devices: Vec<RawDevice>,
}

impl Endpoints {
    pub fn add(
        &mut self,
        direction: &str,
        name: Option<String>,
        description: Option<String>,
        monitor: bool,
    ) -> Result<(), AudioError> {
        self.seen += 1;
        if self.seen > 2 * MAX_DEVICES || (!monitor && self.devices.len() >= MAX_DEVICES) {
            return Err(AudioError::new(
                "inventory_limit",
                "Too many desktop audio devices were reported. Reduce virtual devices and Refresh.",
            ));
        }
        // A monitor is an output loopback, not a microphone. Legitimate virtual
        // sources remain selectable, regardless of names or manufacturers.
        if monitor {
            return Ok(());
        }
        let name = name.filter(|s| !s.is_empty()).ok_or_else(backend_error)?;
        let label = description
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| name.clone());
        self.devices.push(RawDevice {
            native_id: native_id(direction, &name),
            label,
        });
        Ok(())
    }

    fn default_label(&self, direction: &str, name: Option<&str>) -> Option<String> {
        let id = native_id(direction, name?);
        self.devices
            .iter()
            .find(|d| d.native_id == id)
            .map(|d| d.label.clone())
    }
}

fn native_id(direction: &str, name: &str) -> String {
    format!("pulse:{direction}:{name}")
}

pub(super) fn finish(
    inputs: Endpoints,
    outputs: Endpoints,
    initial: Defaults,
    final_defaults: Defaults,
) -> Result<RawInventory, AudioError> {
    if initial != final_defaults {
        return Err(changed_error());
    }
    Ok(RawInventory {
        default_input: inputs.default_label("input", final_defaults.input.as_deref()),
        default_output: outputs.default_label("output", final_defaults.output.as_deref()),
        inputs: inputs.devices,
        outputs: outputs.devices,
    })
}
