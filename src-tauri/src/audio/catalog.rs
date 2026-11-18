//! Session-scoped metadata only. No device or stream handles live in this catalog.
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

pub const INVENTORY_DEADLINE: Duration = Duration::from_secs(5);
pub const MAX_DEVICES: usize = 128;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AudioError {
    pub code: &'static str,
    pub message: &'static str,
    pub cleanup_pending: bool,
}

impl AudioError {
    pub fn new(code: &'static str, message: &'static str) -> Self {
        Self {
            code,
            message,
            cleanup_pending: false,
        }
    }

    fn busy() -> Self {
        Self {
            cleanup_pending: true,
            ..Self::new(
                "inventory_busy",
                "A device scan is still finishing. Try Refresh after it completes.",
            )
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDescriptor {
    pub id: String,
    pub label: String,
    pub available: bool,
    pub is_default: bool,
    pub current_device: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    pub session: String,
    pub revision: u32,
    pub backend: &'static str,
    pub inputs: Vec<DeviceDescriptor>,
    pub outputs: Vec<DeviceDescriptor>,
    pub input_id: String,
    pub output_id: String,
}

pub struct RawDevice {
    pub native_id: String,
    pub label: String,
}

#[derive(Default)]
pub struct RawInventory {
    pub inputs: Vec<RawDevice>,
    pub outputs: Vec<RawDevice>,
    pub default_input: Option<String>,
    pub default_output: Option<String>,
}

#[derive(Clone)]
struct Entry {
    descriptor: DeviceDescriptor,
    native_id: Option<String>,
}

#[derive(Clone, Copy)]
enum Direction {
    Input,
    Output,
}

impl Direction {
    fn parse(value: &str) -> Result<Self, AudioError> {
        match value {
            "input" => Ok(Self::Input),
            "output" => Ok(Self::Output),
            _ => Err(AudioError::new(
                "invalid_direction",
                "Choose an input or output device.",
            )),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }
}

struct Session {
    id: String,
    revision: u32,
    next_handle: u32,
    inputs: Vec<Entry>,
    outputs: Vec<Entry>,
    input_id: String,
    output_id: String,
}

impl Session {
    fn snapshot(&self, backend: &'static str) -> Inventory {
        Inventory {
            session: self.id.clone(),
            revision: self.revision,
            backend,
            inputs: self.inputs.iter().map(|e| e.descriptor.clone()).collect(),
            outputs: self.outputs.iter().map(|e| e.descriptor.clone()).collect(),
            input_id: self.input_id.clone(),
            output_id: self.output_id.clone(),
        }
    }
}

struct Control {
    backend: &'static str,
    next_session: u32,
    next_scan: u32,
    busy: Option<u32>,
    session: Option<Session>,
}

#[derive(Clone)]
pub struct DeviceCatalog(Arc<Mutex<Control>>);

impl DeviceCatalog {
    pub fn new(backend: &'static str) -> Self {
        Self(Arc::new(Mutex::new(Control {
            backend,
            next_session: 0,
            next_scan: 0,
            busy: None,
            session: None,
        })))
    }

    fn lock(&self) -> Result<MutexGuard<'_, Control>, AudioError> {
        self.0.lock().map_err(|_| {
            AudioError::new(
                "internal_error",
                "Audio settings are unavailable. Restart the app.",
            )
        })
    }

    pub fn open_session(&self) -> Result<String, AudioError> {
        let mut state = self.lock()?;
        state.next_session = increment(state.next_session)?;
        let id = format!("audio-session-{}", state.next_session);
        state.session = Some(Session {
            id: id.clone(),
            revision: 0,
            next_handle: 0,
            inputs: vec![],
            outputs: vec![],
            input_id: "input-default".into(),
            output_id: "output-default".into(),
        });
        Ok(id)
    }

    pub fn close_session(&self, id: &str) -> Result<(), AudioError> {
        let mut state = self.lock()?;
        if state.session.as_ref().is_some_and(|s| s.id == id) {
            state.session = None;
        }
        // Do not clear busy: the OS scan may still be running after logout.
        Ok(())
    }

    pub fn validate_session(&self, session: &str) -> Result<(), AudioError> {
        require_session(&mut *self.lock()?, session)?;
        Ok(())
    }

    pub fn shutdown(&self) -> Result<(), AudioError> {
        self.lock()?.session = None;
        Ok(())
    }

    // Native endpoint only; the UI handle cannot select another microphone or
    // bypass a missing device/revision by guessing a backend identifier.
    pub fn input_target(
        &self,
        session: &str,
        revision: u32,
        id: &str,
    ) -> Result<Option<String>, AudioError> {
        self.target(session, revision, id, Direction::Input)
    }

    pub fn output_target(
        &self,
        session: &str,
        revision: u32,
        id: &str,
    ) -> Result<Option<String>, AudioError> {
        self.target(session, revision, id, Direction::Output)
    }

    fn target(
        &self,
        session: &str,
        revision: u32,
        id: &str,
        direction: Direction,
    ) -> Result<Option<String>, AudioError> {
        let mut state = self.lock()?;
        if state.busy.is_some() {
            return Err(AudioError::busy());
        }
        let session = require_session(&mut state, session)?;
        let (entries, selected) = match direction {
            Direction::Input => (&session.inputs, &session.input_id),
            Direction::Output => (&session.outputs, &session.output_id),
        };
        if revision == 0 || revision != session.revision || id != selected {
            return Err(AudioError::new(
                "stale_inventory",
                "The audio selection changed. Refresh and start again.",
            ));
        }
        entries
            .iter()
            .find(|entry| entry.descriptor.id == id && entry.descriptor.available)
            .map(|entry| entry.native_id.clone())
            .ok_or_else(|| {
                AudioError::new(
                    "device_unavailable",
                    "The selected audio device is unavailable. Refresh devices.",
                )
            })
    }

    pub fn begin_refresh(&self, session: &str) -> Result<RefreshPermit, AudioError> {
        let mut state = self.lock()?;
        require_session(&mut state, session)?;
        if state.busy.is_some() {
            return Err(AudioError::busy());
        }
        state.next_scan = increment(state.next_scan)?;
        let ticket = state.next_scan;
        state.busy = Some(ticket);
        Ok(RefreshPermit {
            service: self.clone(),
            session: session.into(),
            ticket,
            started: Instant::now(),
        })
    }

    pub fn select(
        &self,
        session: &str,
        revision: u32,
        direction: &str,
        id: &str,
    ) -> Result<Inventory, AudioError> {
        let mut state = self.lock()?;
        let backend = state.backend;
        let scanning = state.busy.is_some();
        let session = require_session(&mut state, session)?;
        if scanning {
            return Err(AudioError::busy());
        }
        if session.revision != revision || revision == 0 {
            return Err(AudioError::new(
                "stale_inventory",
                "The device list changed. Refresh and select again.",
            ));
        }
        let (entries, selected) = match Direction::parse(direction)? {
            Direction::Input => (&session.inputs, &mut session.input_id),
            Direction::Output => (&session.outputs, &mut session.output_id),
        };
        let entry = entries
            .iter()
            .find(|e| e.descriptor.id == id)
            .ok_or_else(|| {
                AudioError::new(
                    "invalid_device",
                    "This device is not in the current list. Refresh devices.",
                )
            })?;
        if !entry.descriptor.available {
            return Err(AudioError::new(
                "device_unavailable",
                "This device is unavailable. Choose another device or Refresh.",
            ));
        }
        *selected = id.into();
        Ok(session.snapshot(backend))
    }
}

fn increment(value: u32) -> Result<u32, AudioError> {
    value.checked_add(1).ok_or_else(|| {
        AudioError::new(
            "generation_exhausted",
            "Restart the app to reset audio settings.",
        )
    })
}

fn require_session<'a>(state: &'a mut Control, id: &str) -> Result<&'a mut Session, AudioError> {
    state
        .session
        .as_mut()
        .filter(|s| s.id == id)
        .ok_or_else(|| {
            AudioError::new("session_expired", "The audio session ended. Sign in again.")
        })
}

pub struct RefreshPermit {
    service: DeviceCatalog,
    session: String,
    ticket: u32,
    started: Instant,
}

impl RefreshPermit {
    /// Called between native event-loop iterations; never from an audio callback.
    pub fn check(&self) -> Result<Duration, AudioError> {
        let mut state = self.service.lock()?;
        require_session(&mut state, &self.session)?;
        INVENTORY_DEADLINE
            .checked_sub(self.started.elapsed())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| {
                AudioError::new(
                    "inventory_timeout",
                    "The device scan took too long. Try Refresh once it has finished.",
                )
            })
    }

    pub fn run(
        self,
        enumerate: impl FnOnce(&Self) -> Result<RawInventory, AudioError>,
    ) -> Result<Inventory, AudioError> {
        self.check()?;
        let raw = enumerate(&self);
        self.finish(raw)
    }

    pub fn finish(self, raw: Result<RawInventory, AudioError>) -> Result<Inventory, AudioError> {
        let mut state = self.service.lock()?;
        let backend = state.backend;
        let session = require_session(&mut state, &self.session)?;
        if self.started.elapsed() > INVENTORY_DEADLINE {
            return Err(AudioError::new(
                "inventory_timeout",
                "The device scan took too long. Try Refresh once it has finished.",
            ));
        }
        let raw = raw?;
        // Build both lists before committing, so a failed scan preserves all preferences.
        let mut next_handle = session.next_handle;
        let inputs = reconcile(
            Direction::Input,
            raw.inputs,
            raw.default_input,
            &session.inputs,
            &session.input_id,
            &mut next_handle,
        )?;
        let outputs = reconcile(
            Direction::Output,
            raw.outputs,
            raw.default_output,
            &session.outputs,
            &session.output_id,
            &mut next_handle,
        )?;
        let revision = increment(session.revision)?;
        session.inputs = inputs;
        session.outputs = outputs;
        session.next_handle = next_handle;
        session.revision = revision;
        Ok(session.snapshot(backend))
    }
}

impl Drop for RefreshPermit {
    fn drop(&mut self) {
        if let Ok(mut state) = self.service.0.lock() {
            if state.busy == Some(self.ticket) {
                state.busy = None;
            }
        }
    }
}

fn label(text: &str) -> String {
    let clean: String = text.chars().filter(|c| !c.is_control()).take(160).collect();
    if clean.trim().is_empty() {
        "Unnamed audio device".into()
    } else {
        clean
    }
}

fn reconcile(
    direction: Direction,
    mut devices: Vec<RawDevice>,
    default: Option<String>,
    old: &[Entry],
    selected: &str,
    next_handle: &mut u32,
) -> Result<Vec<Entry>, AudioError> {
    if devices.len() > MAX_DEVICES {
        return Err(AudioError::new(
            "inventory_limit",
            "Too many audio devices were reported. Reduce virtual devices and Refresh.",
        ));
    }
    devices.sort_by(|a, b| a.native_id.cmp(&b.native_id));
    devices.dedup_by(|a, b| a.native_id == b.native_id);
    // Normalize once per endpoint, then count/number equal labels in native-ID
    // order. This temporary metadata map is bounded by MAX_DEVICES, off audio callbacks.
    for device in &mut devices {
        device.label = label(&device.label);
    }
    let mut labels = HashMap::<&str, (usize, usize)>::new();
    for device in &devices {
        labels.entry(&device.label).or_default().0 += 1;
    }
    let direction = direction.name();
    let mut entries = vec![Entry {
        descriptor: DeviceDescriptor {
            id: format!("{direction}-default"),
            label: format!("System default {direction}"),
            available: default.is_some(),
            is_default: true,
            current_device: default.map(|s| label(&s)),
        },
        native_id: None,
    }];
    for device in &devices {
        let id = if let Some(previous) = old
            .iter()
            .find(|e| e.native_id.as_ref() == Some(&device.native_id))
        {
            previous.descriptor.id.clone()
        } else {
            *next_handle = increment(*next_handle)?;
            format!("{direction}-{}", *next_handle)
        };
        let name = &device.label;
        let (duplicates, ordinal) = labels.entry(name).or_default();
        *ordinal += 1;
        entries.push(Entry {
            descriptor: DeviceDescriptor {
                id,
                label: if *duplicates > 1 {
                    let suffix = format!(" ({ordinal})");
                    let prefix: String = name.chars().take(160 - suffix.chars().count()).collect();
                    format!("{prefix}{suffix}")
                } else {
                    name.clone()
                },
                available: true,
                is_default: false,
                current_device: None,
            },
            native_id: Some(device.native_id.clone()),
        });
    }
    if !entries.iter().any(|e| e.descriptor.id == selected) {
        if let Some(previous) = old.iter().find(|e| e.descriptor.id == selected) {
            let mut missing = previous.clone();
            missing.descriptor.available = false;
            entries.push(missing);
        }
    }
    Ok(entries)
}

#[cfg(test)]
#[path = "catalog/tests.rs"]
mod tests;
