use super::*;

#[test]
fn output_target_is_independent_of_microphones_and_validates_selection_revision_and_loss() {
    let service = DeviceCatalog::new("test");
    let session = service.open_session().unwrap();
    let inventory = scan(
        &service,
        &session,
        raw(&[], &[("headphones", "Headphones")]),
    );
    assert!(service
        .output_target(&session, inventory.revision, "output-default")
        .unwrap()
        .is_none());
    assert!(service
        .input_target(&session, inventory.revision, "input-default")
        .is_err());
    let output = inventory.outputs.iter().find(|d| !d.is_default).unwrap();
    assert!(service
        .output_target(&session, inventory.revision, &output.id)
        .is_err());
    service
        .select(&session, inventory.revision, "output", &output.id)
        .unwrap();
    assert_eq!(
        service
            .output_target(&session, inventory.revision, &output.id)
            .unwrap()
            .as_deref(),
        Some("headphones")
    );
    assert!(service
        .output_target(&session, inventory.revision + 1, &output.id)
        .is_err());
    let missing = scan(&service, &session, RawInventory::default());
    assert_eq!(
        service
            .output_target(&session, missing.revision, &output.id)
            .unwrap_err()
            .code,
        "device_unavailable"
    );
}

#[test]
fn capture_target_requires_the_current_selected_available_input_and_revision() {
    let service = DeviceCatalog::new("test");
    let session = service.open_session().unwrap();
    let inventory = scan(
        &service,
        &session,
        raw(&[("native-mic", "Mic")], &[("native-out", "Speakers")]),
    );
    assert!(service
        .input_target(&session, inventory.revision, "input-default")
        .unwrap()
        .is_none());
    let mic = inventory.inputs.iter().find(|d| !d.is_default).unwrap();
    assert!(service
        .input_target(&session, inventory.revision, &mic.id)
        .is_err());
    service
        .select(&session, inventory.revision, "input", &mic.id)
        .unwrap();
    assert_eq!(
        service
            .input_target(&session, inventory.revision, &mic.id)
            .unwrap()
            .as_deref(),
        Some("native-mic")
    );
    assert!(service
        .input_target(&session, inventory.revision + 1, &mic.id)
        .is_err());
    let permit = service.begin_refresh(&session).unwrap();
    assert_eq!(
        service
            .input_target(&session, inventory.revision, &mic.id)
            .unwrap_err()
            .code,
        "inventory_busy"
    );
    let missing = permit.finish(Ok(RawInventory::default())).unwrap();
    assert_eq!(
        service
            .input_target(&session, missing.revision, &mic.id)
            .unwrap_err()
            .code,
        "device_unavailable"
    );
    service.close_session(&session).unwrap();
    assert!(service
        .input_target(&session, missing.revision, &mic.id)
        .is_err());
}

#[test]
fn queued_scan_does_not_touch_backend_after_logout_or_deadline() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let permit = service.begin_refresh(&id).unwrap();
    service.close_session(&id).unwrap();
    assert_eq!(
        permit
            .run(|_| panic!("stale scan touched the backend"))
            .unwrap_err()
            .code,
        "session_expired"
    );
    let id = service.open_session().unwrap();
    let mut permit = service.begin_refresh(&id).unwrap();
    permit.started = Instant::now() - INVENTORY_DEADLINE - Duration::from_secs(1);
    assert_eq!(
        permit
            .run(|_| panic!("expired scan touched the backend"))
            .unwrap_err()
            .code,
        "inventory_timeout"
    );
    assert!(service.begin_refresh(&id).is_ok());
}

fn raw(inputs: &[(&str, &str)], outputs: &[(&str, &str)]) -> RawInventory {
    let devices = |pairs: &[(&str, &str)]| {
        pairs
            .iter()
            .map(|(id, name)| RawDevice {
                native_id: id.to_string(),
                label: name.to_string(),
            })
            .collect()
    };
    RawInventory {
        inputs: devices(inputs),
        outputs: devices(outputs),
        default_input: inputs.first().map(|(_, name)| name.to_string()),
        default_output: outputs.first().map(|(_, name)| name.to_string()),
    }
}

fn scan(service: &DeviceCatalog, id: &str, data: RawInventory) -> Inventory {
    service.begin_refresh(id).unwrap().finish(Ok(data)).unwrap()
}

#[test]
fn empty_inventory_keeps_unavailable_default_choices() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let inventory = scan(&service, &id, RawInventory::default());
    assert_eq!(inventory.input_id, "input-default");
    assert_eq!(inventory.output_id, "output-default");
    assert_eq!(inventory.inputs.len(), 1);
    assert!(!inventory.inputs[0].available);
    assert_eq!(
        service
            .select(&id, inventory.revision, "input", "input-default")
            .unwrap_err()
            .code,
        "device_unavailable"
    );
}

#[test]
fn identical_names_have_distinct_handles_and_labels_in_both_directions() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let inventory = scan(
        &service,
        &id,
        raw(
            &[("native-a", "USB"), ("native-b", "USB")],
            &[("native-a", "USB")],
        ),
    );
    assert_ne!(inventory.inputs[1].id, inventory.inputs[2].id);
    assert_ne!(inventory.inputs[1].label, inventory.inputs[2].label);
    assert_ne!(inventory.inputs[1].id, inventory.outputs[1].id);
    assert!(!inventory.inputs[1].id.contains("native-a"));
    assert_eq!(
        service
            .select(&id, inventory.revision, "output", &inventory.inputs[1].id)
            .unwrap_err()
            .code,
        "invalid_device"
    );
}

#[test]
fn explicit_selection_survives_reorder_and_rename_without_following_default() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let first = scan(
        &service,
        &id,
        raw(&[("a", "A"), ("b", "B")], &[("out", "Speaker")]),
    );
    let selected = first.inputs[2].id.clone();
    let changed = service
        .select(&id, first.revision, "input", &selected)
        .unwrap();
    assert_eq!(changed.output_id, "output-default");
    let second = scan(
        &service,
        &id,
        raw(&[("b", "Renamed"), ("a", "A")], &[("out", "Speaker")]),
    );
    assert_eq!(second.input_id, selected);
    assert_eq!(
        second
            .inputs
            .iter()
            .find(|d| d.id == selected)
            .unwrap()
            .label,
        "Renamed"
    );
    assert_eq!(second.inputs[0].current_device.as_deref(), Some("Renamed"));
    assert!(
        !second
            .inputs
            .iter()
            .find(|d| d.id == selected)
            .unwrap()
            .is_default
    );
}

#[test]
fn lost_selection_stays_visible_without_fallback_or_name_matching() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let first = scan(&service, &id, raw(&[("a", "USB")], &[]));
    let selected = first.inputs[1].id.clone();
    service
        .select(&id, first.revision, "input", &selected)
        .unwrap();
    let next = scan(&service, &id, raw(&[("different-id", "USB")], &[]));
    assert_eq!(next.input_id, selected);
    assert!(
        !next
            .inputs
            .iter()
            .find(|d| d.id == selected)
            .unwrap()
            .available
    );
    assert_eq!(
        service
            .select(&id, next.revision, "input", &selected)
            .unwrap_err()
            .code,
        "device_unavailable"
    );
    let returned = scan(&service, &id, raw(&[("a", "USB")], &[]));
    assert_eq!(returned.input_id, selected);
    assert!(
        returned
            .inputs
            .iter()
            .find(|d| d.id == selected)
            .unwrap()
            .available
    );
}

#[test]
fn stale_revisions_and_foreign_sessions_cannot_change_preferences() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let old = scan(&service, &id, raw(&[("a", "A")], &[]));
    let now = scan(&service, &id, raw(&[("a", "A")], &[]));
    assert_eq!(
        service
            .select(&id, old.revision, "input", &old.inputs[1].id)
            .unwrap_err()
            .code,
        "stale_inventory"
    );
    assert_eq!(
        service
            .select("foreign", now.revision, "input", &now.inputs[1].id)
            .unwrap_err()
            .code,
        "session_expired"
    );
    assert_eq!(
        service
            .select(&id, now.revision, "unknown", &now.inputs[1].id)
            .unwrap_err()
            .code,
        "invalid_direction"
    );
}

#[test]
fn only_one_scan_is_admitted_and_logout_does_not_release_its_permit() {
    let service = DeviceCatalog::new("test");
    let old_id = service.open_session().unwrap();
    let permit = service.begin_refresh(&old_id).unwrap();
    assert_eq!(
        service.begin_refresh(&old_id).err().unwrap(),
        AudioError::busy()
    );
    service.close_session(&old_id).unwrap();
    let new_id = service.open_session().unwrap();
    assert_eq!(
        service.begin_refresh(&new_id).err().unwrap().code,
        "inventory_busy"
    );
    assert_eq!(
        permit
            .finish(Ok(raw(&[("old", "Old")], &[])))
            .unwrap_err()
            .code,
        "session_expired"
    );
    let fresh = scan(&service, &new_id, RawInventory::default());
    assert_eq!(fresh.revision, 1);
    assert_eq!(fresh.input_id, "input-default");
}

#[test]
fn late_close_is_idempotent_and_cannot_clear_a_new_session() {
    let service = DeviceCatalog::new("test");
    let old = service.open_session().unwrap();
    let current = service.open_session().unwrap();
    service.close_session(&old).unwrap();
    service.close_session(&old).unwrap();
    assert!(service.begin_refresh(&current).is_ok());
    service.close_session(&current).unwrap();
    assert_eq!(
        service.begin_refresh(&current).err().unwrap().code,
        "session_expired"
    );
}

#[test]
fn timeout_failure_and_abandoned_permits_release_admission_without_overwriting_preferences() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let first = scan(&service, &id, raw(&[("a", "A")], &[]));
    let selected = first.inputs[1].id.clone();
    service
        .select(&id, first.revision, "input", &selected)
        .unwrap();
    let mut permit = service.begin_refresh(&id).unwrap();
    permit.started = Instant::now() - INVENTORY_DEADLINE - Duration::from_secs(1);
    assert_eq!(
        permit.finish(Ok(RawInventory::default())).unwrap_err().code,
        "inventory_timeout"
    );
    let permit = service.begin_refresh(&id).unwrap();
    assert_eq!(
        permit
            .finish(Err(AudioError::new("permission_denied", "Denied")))
            .unwrap_err()
            .code,
        "permission_denied"
    );
    drop(service.begin_refresh(&id).unwrap());
    let next = scan(&service, &id, raw(&[("a", "A")], &[]));
    assert_eq!(next.input_id, selected);
    assert_eq!(next.revision, first.revision + 1);
}

#[test]
fn scans_commit_atomically_and_catalog_size_stays_bounded_during_hotplug() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let first = scan(&service, &id, raw(&[("a", "A")], &[]));
    let too_many = RawInventory {
        outputs: (0..=MAX_DEVICES)
            .map(|i| RawDevice {
                native_id: i.to_string(),
                label: "Out".into(),
            })
            .collect(),
        ..RawInventory::default()
    };
    assert_eq!(
        service
            .begin_refresh(&id)
            .unwrap()
            .finish(Ok(too_many))
            .unwrap_err()
            .code,
        "inventory_limit"
    );
    let selected = service
        .select(&id, first.revision, "input", &first.inputs[1].id)
        .unwrap();
    for i in 0..200 {
        let next = scan(&service, &id, raw(&[(&i.to_string(), "New")], &[]));
        assert_eq!(next.input_id, selected.input_id);
        assert_eq!(next.inputs.len(), 3); // default, currently listed, missing selected
    }
}

#[test]
fn exhaustion_and_empty_labels_are_explicit() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let data = scan(&service, &id, raw(&[("a", "\n\t")], &[]));
    assert_eq!(data.inputs[1].label, "Unnamed audio device");
    service.lock().unwrap().next_session = u32::MAX;
    assert_eq!(
        service.open_session().unwrap_err().code,
        "generation_exhausted"
    );
    assert!(service.begin_refresh(&id).is_ok());
}

#[test]
fn long_unicode_duplicate_labels_remain_bounded_and_distinguishable() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let long = "ă".repeat(200);
    let inventory = scan(&service, &id, raw(&[("a", &long), ("b", &long)], &[]));
    assert_ne!(inventory.inputs[1].label, inventory.inputs[2].label);
    assert!(inventory
        .inputs
        .iter()
        .all(|d| d.label.chars().count() <= 160));
}

#[test]
fn normalized_label_collisions_are_numbered_after_dedup_and_stable_across_reordering() {
    let service = DeviceCatalog::new("test");
    let id = service.open_session().unwrap();
    let long = "ă".repeat(160);
    let truncated = format!("{long}other");
    let mut inputs = [
        ("b", "USB\n"),
        ("a", "USB"),
        ("a", "USB"),
        ("d", "\n"),
        ("c", "\t"),
        ("f", truncated.as_str()),
        ("e", long.as_str()),
    ];
    let first = scan(&service, &id, raw(&inputs, &[]));
    let labels: Vec<_> = first.inputs[1..].iter().map(|d| d.label.as_str()).collect();
    let prefix = "ă".repeat(156);
    assert_eq!(
        labels,
        vec![
            "USB (1)",
            "USB (2)",
            "Unnamed audio device (1)",
            "Unnamed audio device (2)",
            &format!("{prefix} (1)"),
            &format!("{prefix} (2)"),
        ]
    );
    inputs.reverse();
    let next = scan(&service, &id, raw(&inputs, &[]));
    assert_eq!(first.inputs[1..], next.inputs[1..]);
}
