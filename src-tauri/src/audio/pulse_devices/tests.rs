use super::*;
use crate::audio::catalog::{DeviceCatalog, MAX_DEVICES};

fn add(list: &mut Endpoints, direction: &str, name: &str, description: &str, monitor: bool) {
    list.add(
        direction,
        Some(name.into()),
        Some(description.into()),
        monitor,
    )
    .unwrap();
}

#[test]
fn desktop_inventory_keeps_microphones_and_sinks_with_real_default_labels() {
    let mut inputs = Endpoints::default();
    let mut outputs = Endpoints::default();
    // Fixture of desktop roles, not a hardware allowlist in production.
    add(
        &mut inputs,
        "input",
        "headset-mic",
        "SteelSeries Arctis Nova 5 Mono",
        false,
    );
    add(
        &mut inputs,
        "input",
        "built-in-mic",
        "Built-in Audio Analog Stereo",
        false,
    );
    for name in ["hdmi", "headset", "digital"] {
        add(&mut inputs, "input", name, "An output monitor", true);
        add(&mut outputs, "output", name, name, false);
    }
    let defaults = || Defaults {
        input: Some("headset-mic".into()),
        output: Some("headset".into()),
    };
    let raw = inventory::finish(inputs, outputs, defaults(), defaults()).unwrap();
    assert_eq!(raw.inputs.len(), 2);
    assert_eq!(raw.outputs.len(), 3);
    assert_eq!(
        raw.default_input.as_deref(),
        Some("SteelSeries Arctis Nova 5 Mono")
    );
    assert_eq!(raw.default_output.as_deref(), Some("headset"));
    assert_eq!(raw.inputs[0].native_id, "pulse:input:headset-mic");
}

#[test]
fn virtual_sources_and_identical_labels_keep_distinct_native_identities() {
    let mut inputs = Endpoints::default();
    let mut outputs = Endpoints::default();
    add(&mut inputs, "input", "virtual", "Microphone", false);
    add(&mut inputs, "input", "physical", "Microphone", false);
    add(&mut outputs, "output", "virtual", "Microphone", false);
    let raw = inventory::finish(inputs, outputs, Defaults::default(), Defaults::default()).unwrap();
    assert_eq!(raw.inputs.len(), 2);
    assert_ne!(raw.inputs[0].native_id, raw.inputs[1].native_id);
    assert_ne!(raw.inputs[0].native_id, raw.outputs[0].native_id);
    assert_eq!(raw.inputs[0].label, raw.inputs[1].label);
}

#[test]
fn default_monitor_or_missing_endpoint_is_unavailable_without_fallback() {
    for name in ["monitor", "absent"] {
        let mut inputs = Endpoints::default();
        add(&mut inputs, "input", "monitor", "Loopback", true);
        add(&mut inputs, "input", "mic", "Real microphone", false);
        let defaults = || Defaults {
            input: Some(name.into()),
            output: None,
        };
        let raw = inventory::finish(inputs, Endpoints::default(), defaults(), defaults()).unwrap();
        assert!(raw.default_input.is_none());
        assert_eq!(raw.inputs.len(), 1);
        assert!(raw.outputs.is_empty());
    }
    let raw = inventory::finish(
        Endpoints::default(),
        Endpoints::default(),
        Defaults::default(),
        Defaults::default(),
    )
    .unwrap();
    assert!(raw.default_input.is_none());
    assert!(raw.inputs.is_empty());
}

#[test]
fn default_change_rejects_scan_instead_of_committing_partial_selection() {
    let service = DeviceCatalog::new(BACKEND);
    let session = service.open_session().unwrap();
    let result = service.begin_refresh(&session).unwrap().run(|_| {
        inventory::finish(
            Endpoints::default(),
            Endpoints::default(),
            Defaults::default(),
            Defaults {
                input: Some("new".into()),
                output: None,
            },
        )
    });
    assert_eq!(result.unwrap_err().code, "inventory_changed");
    assert!(service.begin_refresh(&session).is_ok());
}

#[test]
fn metadata_limits_include_filtered_monitors_and_names_fall_back_safely() {
    let mut endpoints = Endpoints::default();
    endpoints
        .add("input", Some("endpoint".into()), Some("  ".into()), false)
        .unwrap();
    let raw = inventory::finish(
        endpoints,
        Endpoints::default(),
        Defaults::default(),
        Defaults::default(),
    )
    .unwrap();
    assert_eq!(raw.inputs[0].label, "endpoint");
    assert!(Endpoints::default()
        .add("input", None, Some("No identity".into()), false)
        .is_err());
    let mut endpoints = Endpoints::default();
    for _ in 0..2 * MAX_DEVICES {
        add(&mut endpoints, "input", "monitor", "Monitor", true);
    }
    assert_eq!(
        endpoints.add("input", None, None, true).unwrap_err().code,
        "inventory_limit"
    );
    let mut endpoints = Endpoints::default();
    for n in 0..MAX_DEVICES {
        add(&mut endpoints, "output", &n.to_string(), "Output", false);
    }
    assert_eq!(
        endpoints
            .add("output", Some("extra".into()), None, false)
            .unwrap_err()
            .code,
        "inventory_limit"
    );
}

#[test]
fn callbacks_handle_incomplete_lists_errors_null_metadata_and_bounded_strings() {
    let mut reply = Reply::<Endpoints>::default();
    assert!(!list_end(&mut reply, 0));
    assert!(!reply.done);
    assert!(list_end(&mut reply, -1));
    assert_eq!(reply.error.as_ref().unwrap().code, "backend_error");
    assert!(list_end(&mut reply, 0)); // an error is never overwritten by a later item
    let mut defaults = Reply::<Defaults>::default();
    on_defaults(
        std::ptr::null_mut(),
        std::ptr::null(),
        (&mut defaults as *mut Reply<Defaults>).cast(),
    );
    assert!(defaults.done && defaults.error.is_some());
    let text_value = std::ffi::CString::new("x".repeat(1025)).unwrap();
    // SAFETY: test strings are valid and live until these copies return.
    assert!(unsafe { text(text_value.as_ptr(), 1024) }.is_err());
    assert!(unsafe { text(std::ptr::null(), 1024) }.unwrap().is_none());
    let changed = Cell::new(false);
    on_change(
        std::ptr::null_mut(),
        pulse::PA_SUBSCRIPTION_EVENT_SOURCE,
        0,
        (&changed as *const Cell<bool>).cast_mut().cast(),
    );
    assert!(changed.get());
}

#[test]
#[ignore = "Explicit desktop metadata operation cancellation/cleanup probe"]
fn cancelled_queries_free_userdata_and_never_call_it_later() {
    use std::rc::Rc;
    #[derive(Default)]
    struct Probe {
        _alive: Option<Rc<()>>,
        calls: Rc<Cell<usize>>,
    }
    extern "C" fn on_probe(
        _: *mut pulse::pa_context,
        _: *const pulse::pa_server_info,
        data: *mut c_void,
    ) {
        // SAFETY: Query<Probe> supplies the live userdata until cancellation.
        if let Some(reply) = unsafe { (data as *mut Reply<Probe>).as_mut() } {
            reply.value.calls.set(reply.value.calls.get() + 1);
            reply.done = true;
        }
    }
    let service = DeviceCatalog::new(BACKEND);
    let session = service.open_session().unwrap();
    let scan = service.begin_refresh(&session).unwrap();
    with_context(&scan, None, |context, mainloop| {
        let calls = Rc::new(Cell::new(0));
        for _ in 0..32 {
            let alive = Rc::new(());
            let weak = Rc::downgrade(&alive);
            let mut query = Query::<Probe>::new(context, |data| {
                // SAFETY: same typed callback/Box ownership as production queries.
                unsafe {
                    pulse::pa_context_get_server_info(context.raw.as_ptr(), Some(on_probe), data)
                }
            })?;
            query.reply.value = Probe {
                _alive: Some(alive),
                calls: Rc::clone(&calls),
            };
            drop(query); // pending response; deliberately do not dispatch it
            assert!(weak.upgrade().is_none()); // Box and all captures really dropped
            defaults(context, mainloop, &scan)?; // dispatch subsequent responses
            assert_eq!(calls.get(), 0); // cancelled userdata was never called
        }
        // Exercise the successful Query::wait path too, with identical ownership.
        let alive = Rc::new(());
        let weak = Rc::downgrade(&alive);
        let mut query = Query::<Probe>::new(context, |data| {
            // SAFETY: as above; keep query alive until its successful reply.
            unsafe { pulse::pa_context_get_server_info(context.raw.as_ptr(), Some(on_probe), data) }
        })?;
        query.reply.value = Probe {
            _alive: Some(alive),
            calls: Rc::clone(&calls),
        };
        let result = query.wait(mainloop, &scan)?;
        assert_eq!(calls.get(), 1);
        drop(result);
        assert!(weak.upgrade().is_none());
        Ok(())
    })
    .unwrap();
}

// Explicit metadata recovery uses the desktop service; the two stalled-peer
// probes below use only temporary Unix sockets. No capture, playback or root.
#[test]
#[ignore = "Explicit absent-server error/recovery and desktop metadata probe"]
fn absent_server_is_explicit_and_a_fresh_scan_can_recover() {
    let service = DeviceCatalog::new(BACKEND);
    let session = service.open_session().unwrap();
    let error = service
        .begin_refresh(&session)
        .unwrap()
        .run(|scan| {
            enumerate_at(
                scan,
                Some(c"unix:/od-client-nonexistent-test-server/native"),
            )
        })
        .unwrap_err();
    assert_eq!(error.code, "host_unavailable");
    assert!(!error.cleanup_pending);
    let inventory = service
        .begin_refresh(&session)
        .unwrap()
        .run(enumerate)
        .unwrap();
    assert_eq!(inventory.revision, 1);
}

#[test]
#[ignore = "Explicit native Unix-socket timeout/cleanup probe"]
fn stalled_server_obeys_native_deadline_and_releases_connection() {
    stalled_scan(false);
}

#[test]
#[ignore = "Explicit native Unix-socket logout/cleanup probe"]
fn logout_cancels_stalled_native_scan_and_next_session_can_refresh() {
    stalled_scan(true);
}

fn stalled_scan(logout: bool) {
    use std::{ffi::CString, io::Read, os::unix::net::UnixListener, sync::mpsc, time::Instant};
    let directory = std::env::temp_dir().join(format!(
        "od-client-pulse-test-{}-{}",
        std::process::id(),
        if logout { "logout" } else { "timeout" }
    ));
    std::fs::create_dir(&directory).unwrap();
    let socket = directory.join("native");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = CString::new(format!("unix:{}", socket.display())).unwrap();
    let service = DeviceCatalog::new(BACKEND);
    let session = service.open_session().unwrap();
    let permit = service.begin_refresh(&session).unwrap();
    let (connected, accepted) = mpsc::channel();
    let started = Instant::now();
    std::thread::scope(|scope| {
        let server_task = scope.spawn(|| {
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(started.elapsed() < Duration::from_secs(8));
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("Local test listener failed: {error}"),
                }
            };
            connected.send(()).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(8)))
                .unwrap();
            // Discard handshake bytes without inspection/storage/logging. The
            // fake peer never authenticates/replies; EOF proves client cleanup.
            std::io::copy(&mut stream.by_ref(), &mut std::io::sink()).unwrap();
        });
        let cancel_service = service.clone();
        let cancel_session = session.clone();
        let cancel = scope.spawn(move || {
            accepted.recv_timeout(Duration::from_secs(8)).unwrap();
            if logout {
                cancel_service.close_session(&cancel_session).unwrap();
            }
        });
        let error = permit
            .run(|scan| enumerate_at(scan, Some(&server)))
            .unwrap_err();
        assert_eq!(
            error.code,
            if logout {
                "session_expired"
            } else {
                "inventory_timeout"
            }
        );
        assert!(started.elapsed() < Duration::from_secs(if logout { 2 } else { 7 }));
        cancel.join().unwrap();
        server_task.join().unwrap();
    });
    // Reuse admission after the native objects have actually dropped.
    let next = service.open_session().unwrap();
    assert!(service.begin_refresh(&next).is_ok());
    std::fs::remove_file(socket).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
