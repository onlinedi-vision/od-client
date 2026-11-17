use super::*;
use std::sync::atomic::AtomicUsize;

fn format() -> CaptureFormat {
    CaptureFormat {
        sample_rate: 48000,
        channels: 1,
        sample_format: "f32",
        buffer_frames: Some(480),
    }
}
fn job(controller: &Controller, owner: u32, intent: u32) -> Job {
    let mut queued = None;
    controller
        .start(owner, intent, Some("native-input".into()), |work| {
            queued = Some(work)
        })
        .unwrap();
    queued.unwrap()
}

#[test]
fn stop_before_dispatch_never_touches_native_audio_and_blocks_late_start() {
    let controller = Controller::default();
    let work = job(&controller, 1, 1);
    assert_eq!(controller.stop(1, 2, false).unwrap().phase, "stopping");
    assert!(controller
        .start(1, 1, None, |_| panic!("late Start admitted"))
        .is_err());
    assert!(controller.busy().unwrap());
    work.execute(|_, _| panic!("cancelled job touched native audio"));
    assert!(!controller.busy().unwrap());
    assert_eq!(controller.snapshot(1).unwrap().phase, "idle");
}

#[test]
fn fatal_callback_fault_gates_all_modes_before_worker_poll_and_preserves_its_cause() {
    for mode in [Mode::Input, Mode::Output, Mode::Loopback] {
        for (fault, code) in [
            (1, "capture_failed"),
            (2, "permission_denied"),
            (3, "device_unavailable"),
            (4, "capture_timing_unavailable"),
        ] {
            let controller = Controller::default();
            let mut work = None;
            controller
                .start_test(
                    1,
                    1,
                    Selection {
                        mode,
                        input: None,
                        output: None,
                    },
                    |job| work = Some(job),
                )
                .unwrap();
            let work = work.unwrap();
            work.run.started(format()).unwrap();
            work.run.meter.fail(fault);
            assert!(
                !work.run.enabled(),
                "callback audio continued after fault {fault}"
            );
            assert_eq!(work.run.check().unwrap_err().code, code);
            let stopping = controller.snapshot(1).unwrap();
            assert!(stopping.cleanup_pending);
            assert_eq!(stopping.level, 0);
            drop(work);
            let stopped = controller.snapshot(1).unwrap();
            assert!(!stopped.cleanup_pending);
            assert_eq!(stopped.phase, "failed");
            assert_eq!(stopped.error.unwrap().code, code);
        }
    }
}

#[test]
fn repeated_start_stop_are_idempotent_and_only_one_job_is_admitted() {
    let controller = Controller::default();
    let work = job(&controller, 1, 1);
    controller
        .start(1, 1, None, |_| panic!("duplicate job"))
        .unwrap();
    controller
        .start(1, 2, None, |_| panic!("parallel job"))
        .unwrap();
    controller.stop(1, 3, false).unwrap();
    controller.stop(1, 3, false).unwrap();
    assert_eq!(
        controller
            .start(1, 4, None, |_| panic!("cleanup overlap"))
            .unwrap_err()
            .code,
        "capture_busy"
    );
    drop(work);
    let work = job(&controller, 1, 5);
    drop(work);
    assert!(!controller.busy().unwrap());
}

#[test]
fn late_build_after_stop_drops_resource_without_play_and_admission_waits_for_cleanup() {
    let controller = Controller::default();
    let work = job(&controller, 1, 1);
    let played = AtomicUsize::new(0);
    let dropped = AtomicUsize::new(0);
    struct Resource<'a>(&'a AtomicUsize);
    impl Drop for Resource<'_> {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    let (entered, built) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        let dropped = &dropped;
        let played = &played;
        let task = scope.spawn(move || {
            work.execute(|run, target| {
                assert_eq!(target, Some("native-input"));
                let _resource = Resource(dropped);
                entered.send(()).unwrap();
                released.recv_timeout(Duration::from_secs(2)).unwrap();
                run.check()?;
                played.fetch_add(1, Ordering::Relaxed);
                run.started(format())
            })
        });
        built.recv_timeout(Duration::from_secs(2)).unwrap();
        controller.stop(1, 2, false).unwrap();
        assert!(controller.busy().unwrap());
        assert!(controller
            .start(1, 3, None, |_| panic!("second OS job"))
            .is_err());
        release.send(()).unwrap();
        task.join().unwrap();
    });
    assert_eq!(played.load(Ordering::Relaxed), 0);
    assert_eq!(dropped.load(Ordering::Relaxed), 1);
    assert_eq!(controller.snapshot(1).unwrap().phase, "idle");
}

#[test]
fn close_is_final_for_owner_and_old_panel_cannot_stop_new_panel() {
    let controller = Controller::default();
    let old = job(&controller, 1, 1);
    controller.stop(1, 2, true).unwrap();
    controller.stop(1, 2, true).unwrap();
    assert!(controller
        .start(1, 3, None, |_| panic!("closed owner reopened"))
        .is_err());
    drop(old);
    let new = job(&controller, 2, 1);
    assert!(controller.stop(1, 4, true).is_err());
    assert!(new.run.enabled());
    drop(new);
}

#[test]
fn successful_running_reports_only_latest_rate_limited_summary_and_zero_after_stop() {
    let controller = Controller::default();
    let work = job(&controller, 1, 1);
    work.run.started(format()).unwrap();
    work.run.meter.samples([1.0; 480].into_iter());
    let first = controller.snapshot(1).unwrap();
    assert_eq!(first.phase, "running");
    assert_eq!(first.level, 100);
    assert_eq!(first.format, Some(format()));
    assert_eq!(first.callback_frames, Some(480));
    assert_eq!(controller.snapshot(1).unwrap().level, 100); // cached within 100 ms
    lock(&controller.0).unwrap().sampled = Some(Instant::now() - LEVEL_INTERVAL);
    assert_eq!(controller.snapshot(1).unwrap().level, 0); // no old level replay
    controller.stop(1, 2, false).unwrap();
    assert_eq!(controller.snapshot(1).unwrap().level, 0);
    assert!(work.run.started(format()).is_err()); // late ready cannot reactivate
    drop(work);
}

fn expire(work: &mut Job, running: bool) {
    let mut state = lock(&work.state).unwrap();
    state.active = None;
    let run = Arc::get_mut(&mut work.run).unwrap();
    run.created =
        Instant::now() - if running { TEST_LIMIT } else { START_LIMIT } - Duration::from_secs(1);
    run.running.store(running, Ordering::Relaxed);
    if running {
        state.snapshot.phase = "running";
    }
    state.active = Some(Arc::clone(&work.run));
}

#[test]
fn native_start_deadline_remains_visible_after_cleanup_and_never_plays_late() {
    let controller = Controller::default();
    let mut work = job(&controller, 1, 1);
    expire(&mut work, false);
    assert_eq!(work.run.check().unwrap_err().code, "capture_timeout");
    let snapshot = controller.snapshot(1).unwrap();
    assert!(snapshot.cleanup_pending);
    assert_eq!(snapshot.error.unwrap().code, "capture_timeout");
    assert_eq!(
        controller.snapshot(1).unwrap().error.unwrap().code,
        "capture_timeout"
    );
    work.execute(|_, _| panic!("expired start touched native audio"));
    let snapshot = controller.snapshot(1).unwrap();
    assert_eq!(snapshot.phase, "failed");
    assert!(!snapshot.cleanup_pending);
    assert_eq!(snapshot.error.unwrap().code, "capture_timeout");
}

#[test]
fn native_30_second_limit_ends_normally_independent_of_ui_timer() {
    let controller = Controller::default();
    let mut work = job(&controller, 1, 1);
    expire(&mut work, true);
    assert_eq!(work.run.check().unwrap_err().code, "test_finished");
    assert!(controller.snapshot(1).unwrap().cleanup_pending);
    drop(work);
    let snapshot = controller.snapshot(1).unwrap();
    assert_eq!(snapshot.phase, "idle");
    assert_eq!(snapshot.ended, Some("duration_limit"));
    assert!(snapshot.error.is_none());
}

#[test]
fn native_failure_and_callback_permission_loss_preserve_error_and_allow_explicit_recovery() {
    let controller = Controller::default();
    let work = job(&controller, 1, 1);
    work.execute(|_, _| Err(AudioError::new("permission_denied", "Permission denied")));
    assert_eq!(
        controller.snapshot(1).unwrap().error.unwrap().code,
        "permission_denied"
    );
    let work = job(&controller, 1, 2);
    work.run.started(format()).unwrap();
    work.run.meter.fail(2);
    let snapshot = controller.snapshot(1).unwrap();
    assert_eq!(snapshot.phase, "stopping");
    assert_eq!(snapshot.error.unwrap().code, "permission_denied");
    drop(work);
    assert_eq!(controller.snapshot(1).unwrap().phase, "failed");
    assert!(!controller.busy().unwrap());
}

#[test]
fn fifty_cycles_release_every_job_and_old_results_cannot_change_new_state() {
    let controller = Controller::default();
    for owner in 1..=50 {
        let work = job(&controller, owner, 1);
        work.run.started(format()).unwrap();
        controller.stop(owner, 2, true).unwrap();
        assert!(work.run.started(format()).is_err());
        drop(work);
        assert!(!controller.busy().unwrap());
        assert_eq!(controller.snapshot(owner).unwrap().level, 0);
    }
}

#[test]
fn reset_and_controller_drop_cancel_inflight_work_without_a_late_native_call() {
    let controller = Controller::default();
    let old = job(&controller, 1, 1);
    controller.reset().unwrap();
    assert!(!old.run.enabled());
    assert!(controller.busy().unwrap());
    drop(old);
    let new = job(&controller, 1, 1);
    drop(controller);
    new.execute(|_, _| panic!("service drop left capture enabled"));
}

fn playback_job(controller: &Controller, intent: u32, mode: Mode) -> Job {
    let mut queued = None;
    controller
        .start_test(
            1,
            intent,
            Selection {
                mode,
                input: (mode == Mode::Loopback).then(|| "mic".into()),
                output: Some("headphones".into()),
            },
            |work| queued = Some(work),
        )
        .unwrap();
    queued.unwrap()
}

#[test]
fn tone_needs_only_output_and_modes_share_one_admission_until_cleanup() {
    let controller = Controller::default();
    let tone = playback_job(&controller, 1, Mode::Output);
    assert!(tone.selection.input.is_none());
    assert_eq!(tone.selection.output.as_deref(), Some("headphones"));
    assert_eq!(
        controller
            .start_test(
                1,
                2,
                Selection {
                    mode: Mode::Loopback,
                    input: None,
                    output: None,
                },
                |_| panic!("parallel loopback")
            )
            .unwrap_err()
            .code,
        "test_busy"
    );
    tone.run.started_playback(None, format()).unwrap();
    assert_eq!(controller.snapshot(1).unwrap().mode, Mode::Output);
    controller.stop(1, 3, false).unwrap();
    assert_eq!(
        controller
            .start_test(
                1,
                4,
                Selection {
                    mode: Mode::Input,
                    input: None,
                    output: None,
                },
                |_| panic!("overlapping cleanup")
            )
            .unwrap_err()
            .code,
        "capture_busy"
    );
    drop(tone);
    let loopback = playback_job(&controller, 5, Mode::Loopback);
    loopback
        .run
        .started_playback(
            Some(format()),
            CaptureFormat {
                channels: 2,
                ..format()
            },
        )
        .unwrap();
    let state = controller.snapshot(1).unwrap();
    assert_eq!(state.mode, Mode::Loopback);
    assert_eq!(state.format.unwrap().channels, 1);
    assert_eq!(state.output_format.unwrap().channels, 2);
    drop(loopback);
}

#[test]
fn tone_deadline_starts_at_activation_but_loopback_includes_setup_time() {
    for mode in [Mode::Output, Mode::Loopback] {
        let controller = Controller::default();
        let mut work = playback_job(&controller, 1, mode);
        {
            let mut state = lock(&controller.0).unwrap();
            state.active = None;
            Arc::get_mut(&mut work.run).unwrap().created = Instant::now() - Duration::from_secs(2);
            state.active = Some(Arc::clone(&work.run));
        }
        work.run
            .started_playback((mode == Mode::Loopback).then(format), format())
            .unwrap();
        let remaining = work.run.check().unwrap();
        if mode == Mode::Output {
            assert!(remaining > Duration::from_millis(900) && remaining <= Duration::from_secs(1));
            work.run.activated.store(0, Ordering::Release);
            assert_eq!(work.run.check().unwrap_err().code, "test_finished");
            work.execute_test(|run, _| run.check().map(|_| ()));
            assert_eq!(
                controller.snapshot(1).unwrap().ended,
                Some("duration_limit")
            );
        } else {
            assert!(remaining <= Duration::from_secs(28) && remaining > Duration::from_secs(27));
            expire(&mut work, true);
            assert_eq!(work.run.check().unwrap_err().code, "test_finished");
        }
    }
}

#[test]
fn setup_retry_cannot_reenable_stopped_running_or_expired_streams() {
    let controller = Controller::default();
    let work = playback_job(&controller, 1, Mode::Loopback);
    work.run.quiet(); // all previous native handles have returned from Drop
    assert!(!work.run.enabled());
    work.run.retry_setup().unwrap();
    assert!(work.run.enabled());
    work.run.started_playback(Some(format()), format()).unwrap();
    work.run.quiet();
    assert!(work.run.retry_setup().is_err());
    assert!(!work.run.enabled());
    controller.stop(1, 2, false).unwrap();
    assert!(work.run.retry_setup().is_err());
    drop(work);
    let mut late = playback_job(&controller, 3, Mode::Output);
    expire(&mut late, false);
    late.run.quiet();
    assert_eq!(late.run.retry_setup().unwrap_err().code, "capture_timeout");
    assert!(!late.run.enabled());
    late.execute_test(|_, _| panic!("retry exceeded original start deadline"));
}

#[test]
fn second_stream_failure_gates_both_callbacks_before_drops_and_preserves_cause() {
    let controller = Controller::default();
    let work = playback_job(&controller, 1, Mode::Loopback);
    let dropped = AtomicUsize::new(0);
    struct Duplex<'a> {
        run: Arc<Run>,
        dropped: &'a AtomicUsize,
        controller: &'a Controller,
    }
    impl Drop for Duplex<'_> {
        fn drop(&mut self) {
            self.run.quiet();
            assert!(!self.run.enabled());
            assert!(self.controller.busy().unwrap());
            self.dropped.fetch_add(2, Ordering::Relaxed);
        }
    }
    work.execute_test(|run, selection| {
        assert_eq!(selection.mode, Mode::Loopback);
        let _streams = Duplex {
            run: Arc::clone(run),
            dropped: &dropped,
            controller: &controller,
        };
        Err(AudioError::new("device_unavailable", "Output was removed"))
    });
    assert_eq!(dropped.load(Ordering::Relaxed), 2);
    assert!(!controller.busy().unwrap());
    assert_eq!(
        controller.snapshot(1).unwrap().error.unwrap().code,
        "device_unavailable"
    );
}
