//! Desktop audio. Inventory never opens streams; capture lives in its explicit
//! child adapter. All libpulse objects/callbacks stay on their owning worker.
//! No CLI subprocesses, daemon autospawn, or permanent device polling.
use super::{
    budget::Budget,
    catalog::{AudioError, RawInventory, RefreshPermit},
};
use libpulse_binding::{
    mainloop::{
        api::{Mainloop as MainloopApi, MainloopInner},
        events::timer::TimeEvent,
        standard::{Mainloop, MainloopInternal},
    },
    time::MonotonicTs,
};
use libpulse_sys as pulse;
use std::time::Duration;
use std::{cell::Cell, ffi::CStr, marker::PhantomData, os::raw::c_void, ptr::NonNull, rc::Rc};

mod capture;
mod inventory;
mod playback;
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
use inventory::{Defaults, Endpoints};

pub const BACKEND: &str = "PulseAudio";
const POLL_SLICE: Duration = Duration::from_millis(50);

// One reusable native timer bounds poll during this scan only. It avoids the
// historical ms/us ambiguity of pa_mainloop_prepare(timeout) before PA 14.
struct EventLoop {
    timer: TimeEvent<MainloopInner<MainloopInternal>>,
    inner: Mainloop,
}

impl EventLoop {
    fn new() -> Result<Self, AudioError> {
        let mut inner = Mainloop::new().ok_or_else(backend_error)?;
        let timer = inner
            .new_timer_event_rt(MonotonicTs::now() + POLL_SLICE, Box::new(|_| {}))
            .ok_or_else(backend_error)?;
        Ok(Self { timer, inner })
    }
}

fn backend_error() -> AudioError {
    AudioError::new(
        "backend_error",
        "Desktop audio devices could not be read. Check the audio service and Refresh.",
    )
}

fn changed_error() -> AudioError {
    AudioError::new(
        "inventory_changed",
        "Desktop audio devices changed during the scan. Refresh devices.",
    )
}

// Owns both the context and its subscription userdata. The mainloop outlives it.
struct Context {
    raw: NonNull<pulse::pa_context>,
    changed: Box<Cell<bool>>,
    _thread: PhantomData<Rc<()>>,
}

impl Context {
    fn error(&self) -> AudioError {
        // SAFETY: raw is an owned, live context; no other thread accesses it.
        let code = unsafe { pulse::pa_context_errno(self.raw.as_ptr()) };
        if code == pulse::PA_ERR_ACCESS as i32 {
            AudioError::new(
                "permission_denied",
                "Desktop audio access was denied. Check the audio service permissions.",
            )
        } else if code == pulse::PA_ERR_BUSY as i32 {
            AudioError::new(
                "device_busy",
                "The microphone is busy. Close its other user and try again.",
            )
        } else if code == pulse::PA_ERR_NOENTITY as i32 {
            super::capture::unavailable()
        } else if code == pulse::PA_ERR_NOTSUPPORTED as i32 {
            super::capture::unsupported()
        } else {
            AudioError::new("host_unavailable", "Desktop audio is unavailable. Start PulseAudio or PipeWire with its PulseAudio service, then Refresh.")
        }
    }

    fn state(&self) -> pulse::pa_context_state_t {
        // SAFETY: context remains live until Drop, on this same worker.
        unsafe { pulse::pa_context_get_state(self.raw.as_ptr()) }
    }

    fn check(&self) -> Result<(), AudioError> {
        if self.changed.get() {
            Err(changed_error())
        } else if matches!(
            self.state(),
            pulse::PA_CONTEXT_FAILED | pulse::PA_CONTEXT_TERMINATED
        ) {
            Err(self.error())
        } else {
            Ok(())
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // SAFETY: queries have already dropped; no callback is executing here.
        // Unregister before freeing userdata, disconnect before freeing mainloop.
        unsafe {
            pulse::pa_context_set_subscribe_callback(self.raw.as_ptr(), None, std::ptr::null_mut());
            pulse::pa_context_disconnect(self.raw.as_ptr());
            pulse::pa_context_unref(self.raw.as_ptr());
        }
    }
}

#[derive(Default)]
struct Reply<T> {
    value: T,
    done: bool,
    error: Option<AudioError>,
}

// libpulse-binding 2.30.1's Operation::from_raw loses the non-null callback
// pointer on cancellation. Use the approved sys API here: Rust owns userdata
// throughout, and pa_operation_cancel disables callbacks before it is freed.
struct Query<'a, T> {
    raw: NonNull<pulse::pa_operation>,
    reply: Box<Reply<T>>,
    context: &'a Context,
}

impl<'a, T: Default> Query<'a, T> {
    fn new(
        context: &'a Context,
        start: impl FnOnce(*mut c_void) -> *mut pulse::pa_operation,
    ) -> Result<Self, AudioError> {
        if context.state() != pulse::PA_CONTEXT_READY {
            return Err(context.error());
        }
        let mut reply = Box::<Reply<T>>::default();
        let userdata = (&mut *reply as *mut Reply<T>).cast();
        // start registers one of our typed callbacks. The Box address is stable.
        let raw = NonNull::new(start(userdata)).ok_or_else(|| context.error())?;
        Ok(Self {
            raw,
            reply,
            context,
        })
    }

    fn wait(mut self, mainloop: &mut EventLoop, scan: &impl Budget) -> Result<T, AudioError> {
        loop {
            scan.check()?;
            self.context.check()?;
            if let Some(error) = self.reply.error.take() {
                return Err(error);
            }
            // SAFETY: the operation is live and owned exclusively by this query.
            match unsafe { pulse::pa_operation_get_state(self.raw.as_ptr()) } {
                pulse::PA_OPERATION_DONE if self.reply.done => {
                    return Ok(std::mem::take(&mut self.reply.value))
                }
                pulse::PA_OPERATION_RUNNING => step(mainloop, scan)?,
                _ => return Err(backend_error()),
            }
        }
    }
}

impl<T> Drop for Query<'_, T> {
    fn drop(&mut self) {
        // SAFETY: called outside dispatch/callbacks, context is still live.
        // C never owns our Box. Cancel first on every path, then unref, then the
        // reply Box (including partial lists) is dropped by Rust exactly once.
        unsafe {
            pulse::pa_operation_cancel(self.raw.as_ptr());
            pulse::pa_operation_unref(self.raw.as_ptr());
        }
    }
}

fn step(mainloop: &mut EventLoop, scan: &impl Budget) -> Result<(), AudioError> {
    let wait = scan.check()?.min(POLL_SLICE);
    mainloop.timer.restart_rt(MonotonicTs::now() + wait);
    // The monotonic timer bounds the poll; no numeric timeout conversion or
    // allocation on each iteration. libpulse may wake sooner for socket events.
    mainloop.inner.prepare(None).map_err(|_| backend_error())?;
    mainloop.inner.poll().map_err(|_| backend_error())?;
    mainloop.inner.dispatch().map_err(|_| backend_error())?;
    scan.check()?;
    Ok(())
}

extern "C" fn on_change(
    _: *mut pulse::pa_context,
    _: pulse::pa_subscription_event_type_t,
    _: u32,
    userdata: *mut c_void,
) {
    // SAFETY: registered only with Context::changed's stable Box, unregistered
    // before Drop frees it. Callbacks run synchronously inside this worker.
    if let Some(changed) = unsafe { (userdata as *const Cell<bool>).as_ref() } {
        changed.set(true);
    }
}

extern "C" fn on_subscribed(_: *mut pulse::pa_context, success: i32, userdata: *mut c_void) {
    // SAFETY: Query<()> supplies this stable, live Reply until cancellation.
    if let Some(reply) = unsafe { (userdata as *mut Reply<()>).as_mut() } {
        reply.done = true;
        if success == 0 {
            reply.error = Some(backend_error());
        }
    }
}

// SAFETY at call sites: text is either null or a NUL-terminated libpulse string
// valid for this callback. Copy only bounded metadata; never store C pointers.
unsafe fn text(
    ptr: *const std::os::raw::c_char,
    limit: usize,
) -> Result<Option<String>, AudioError> {
    if ptr.is_null() {
        return Ok(None);
    }
    let bytes = unsafe { CStr::from_ptr(ptr) }.to_bytes();
    if bytes.len() > limit {
        return Err(backend_error());
    }
    Ok(Some(String::from_utf8_lossy(bytes).into_owned()))
}

extern "C" fn on_defaults(
    _: *mut pulse::pa_context,
    info: *const pulse::pa_server_info,
    userdata: *mut c_void,
) {
    // SAFETY: libpulse supplies info for this callback; Query<Defaults> owns reply.
    let Some(reply) = (unsafe { (userdata as *mut Reply<Defaults>).as_mut() }) else {
        return;
    };
    reply.done = true;
    reply.error = (|| {
        let info = unsafe { info.as_ref() }.ok_or_else(backend_error)?;
        reply.value.input = unsafe { text(info.default_source_name, 1024) }?;
        reply.value.output = unsafe { text(info.default_sink_name, 1024) }?;
        Ok::<_, AudioError>(())
    })()
    .err();
}

extern "C" fn on_source(
    _: *mut pulse::pa_context,
    info: *const pulse::pa_source_info,
    end: i32,
    userdata: *mut c_void,
) {
    // SAFETY: Query<Endpoints> owns reply; info is borrowed only during callback.
    let Some(reply) = (unsafe { (userdata as *mut Reply<Endpoints>).as_mut() }) else {
        return;
    };
    if list_end(reply, end) {
        return;
    }
    reply.error = (|| {
        let info = unsafe { info.as_ref() }.ok_or_else(backend_error)?;
        reply.value.add(
            "input",
            unsafe { text(info.name, 1024) }?,
            unsafe { text(info.description, 4096) }?,
            info.monitor_of_sink != pulse::PA_INVALID_INDEX,
        )
    })()
    .err();
}

extern "C" fn on_sink(
    _: *mut pulse::pa_context,
    info: *const pulse::pa_sink_info,
    end: i32,
    userdata: *mut c_void,
) {
    // SAFETY: as on_source, but registered only for sink info.
    let Some(reply) = (unsafe { (userdata as *mut Reply<Endpoints>).as_mut() }) else {
        return;
    };
    if list_end(reply, end) {
        return;
    }
    reply.error = (|| {
        let info = unsafe { info.as_ref() }.ok_or_else(backend_error)?;
        reply.value.add(
            "output",
            unsafe { text(info.name, 1024) }?,
            unsafe { text(info.description, 4096) }?,
            false,
        )
    })()
    .err();
}

fn list_end(reply: &mut Reply<Endpoints>, end: i32) -> bool {
    if reply.error.is_some() {
        return true;
    }
    if end != 0 {
        reply.done = true;
        if end < 0 {
            reply.error = Some(backend_error());
        }
        return true;
    }
    false
}

fn defaults(
    context: &Context,
    mainloop: &mut EventLoop,
    scan: &impl Budget,
) -> Result<Defaults, AudioError> {
    Query::new(context, |data| {
        // SAFETY: live Ready context, correctly typed callback and Query userdata.
        unsafe { pulse::pa_context_get_server_info(context.raw.as_ptr(), Some(on_defaults), data) }
    })?
    .wait(mainloop, scan)
}

pub fn enumerate(scan: &RefreshPermit) -> Result<RawInventory, AudioError> {
    enumerate_at(scan, None)
}

// An explicit server is used only by native failure tests, never supplied by Vue.
fn enumerate_at(scan: &impl Budget, server: Option<&CStr>) -> Result<RawInventory, AudioError> {
    with_context(scan, server, |context, mainloop| {
        scan_inventory(context, mainloop, scan)
    })
}

fn with_context<T>(
    scan: &impl Budget,
    server: Option<&CStr>,
    read: impl FnOnce(&Context, &mut EventLoop) -> Result<T, AudioError>,
) -> Result<T, AudioError> {
    scan.check()?;
    if libpulse_binding::version::library_version_is_too_old() != Ok(false) {
        return Err(AudioError::new(
            "unsupported_audio_version",
            "Desktop audio requires libpulse 8 or newer.",
        ));
    }
    let mut mainloop = EventLoop::new()?;
    // SAFETY: binding exposes the exact sys vtable; fixed name contains a NUL.
    // Context is declared after mainloop and drops first on every return path.
    let raw = unsafe {
        pulse::pa_context_new(
            mainloop.inner.get_api().as_ref(),
            c"od-client audio settings".as_ptr(),
        )
    };
    let context = Context {
        raw: NonNull::new(raw).ok_or_else(backend_error)?,
        changed: Box::new(Cell::new(false)),
        _thread: PhantomData,
    };
    // SAFETY: pointers remain live until disconnect; NOAUTOSPAWN never starts a daemon.
    let connected = unsafe {
        pulse::pa_context_connect(
            context.raw.as_ptr(),
            server.map_or(std::ptr::null(), CStr::as_ptr),
            pulse::PA_CONTEXT_NOAUTOSPAWN,
            std::ptr::null(),
        )
    };
    if connected < 0 {
        return Err(context.error());
    }
    while context.state() != pulse::PA_CONTEXT_READY {
        context.check()?;
        step(&mut mainloop, scan)?;
    }
    read(&context, &mut mainloop)
}

fn scan_inventory(
    context: &Context,
    mainloop: &mut EventLoop,
    scan: &impl Budget,
) -> Result<RawInventory, AudioError> {
    // Watch only this short scan. Any source/sink/default mutation invalidates
    // the entire result; no retries, subscription, or connection survive return.
    unsafe {
        pulse::pa_context_set_subscribe_callback(
            context.raw.as_ptr(),
            Some(on_change),
            (&*context.changed as *const Cell<bool>).cast_mut().cast(),
        );
    }
    Query::<()>::new(context, |data| {
        // SAFETY: same context/callback ownership as above, no stream operations.
        unsafe {
            pulse::pa_context_subscribe(
                context.raw.as_ptr(),
                pulse::PA_SUBSCRIPTION_MASK_SOURCE
                    | pulse::PA_SUBSCRIPTION_MASK_SINK
                    | pulse::PA_SUBSCRIPTION_MASK_SERVER,
                Some(on_subscribed),
                data,
            )
        }
    })?
    .wait(mainloop, scan)?;
    let initial = defaults(context, mainloop, scan)?;
    let inputs = Query::<Endpoints>::new(context, |data| {
        // SAFETY: callback type and userdata match; context remains Ready.
        unsafe {
            pulse::pa_context_get_source_info_list(context.raw.as_ptr(), Some(on_source), data)
        }
    })?
    .wait(mainloop, scan)?;
    let mut outputs = Query::<Endpoints>::new(context, |data| {
        // SAFETY: as above for the independent output list.
        unsafe { pulse::pa_context_get_sink_info_list(context.raw.as_ptr(), Some(on_sink), data) }
    })?;
    outputs.reply.value.seen = inputs.seen;
    let outputs = outputs.wait(mainloop, scan)?;
    let final_defaults = defaults(context, mainloop, scan)?;
    context.check()?;
    scan.check()?;
    inventory::finish(inputs, outputs, initial, final_defaults)
}

#[cfg(test)]
mod tests;
