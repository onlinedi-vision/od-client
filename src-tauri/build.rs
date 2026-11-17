fn main() {
    let audio = tauri_build::InlinedPlugin::new()
        .commands(&[
            "audio_open_session",
            "audio_close_session",
            "audio_refresh_devices",
            "audio_select_device",
            "audio_capture_start",
            "audio_capture_stop",
            "audio_capture_state",
        ])
        .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands);
    tauri_build::try_build(tauri_build::Attributes::new().plugin("audio", audio))
        .expect("Tauri build failed");
}
