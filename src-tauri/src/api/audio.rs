use crate::audio::{self, AudioError, AudioService, CaptureSnapshot, Inventory};
use tauri::{Manager, State, WebviewWindow};

// Inline plugin gives these commands a separate local-only Tauri capability,
// without enabling an application-wide ACL for the existing chat commands.
pub(crate) fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("audio")
        .invoke_handler(tauri::generate_handler![
            audio_open_session,
            audio_close_session,
            audio_refresh_devices,
            audio_select_device,
            audio_capture_start,
            audio_capture_stop,
            audio_capture_state,
        ])
        .on_event(|app, event| {
            let closing = matches!(event, tauri::RunEvent::WindowEvent { label, event: tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed, .. } if label == "main");
            if closing || matches!(event, tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit) {
                app.state::<AudioService>().shutdown();
            }
        })
        .build()
}

#[tauri::command]
// Keep the existing local IPC arguments flat; this command only delegates.
#[allow(clippy::too_many_arguments)]
pub(crate) fn audio_capture_start(
    window: WebviewWindow,
    service: State<'_, AudioService>,
    session: String,
    owner: u32,
    intent: u32,
    revision: u32,
    input_id: String,
    mode: Option<audio::Mode>,
    output_id: Option<String>,
) -> Result<CaptureSnapshot, AudioError> {
    require_local(&window)?;
    service.start_capture(
        &session,
        owner,
        intent,
        audio::Start {
            revision,
            input_id,
            output_id,
            mode: mode.unwrap_or_default(),
        },
    )
}

#[tauri::command]
pub(crate) fn audio_capture_stop(
    window: WebviewWindow,
    service: State<'_, AudioService>,
    session: String,
    owner: u32,
    intent: u32,
    close: bool,
) -> Result<CaptureSnapshot, AudioError> {
    require_local(&window)?;
    service.stop_capture(&session, owner, intent, close)
}

#[tauri::command]
pub(crate) fn audio_capture_state(
    window: WebviewWindow,
    service: State<'_, AudioService>,
    session: String,
    owner: u32,
) -> Result<CaptureSnapshot, AudioError> {
    require_local(&window)?;
    service.capture_state(&session, owner)
}

// App commands are available only to the main local UI. This is an application
// lifecycle session, not a substitute for backend authentication or membership.
fn require_local(window: &WebviewWindow) -> Result<(), AudioError> {
    let url = window.url().map_err(|_| {
        AudioError::new(
            "invalid_window",
            "Audio settings require the local app window.",
        )
    })?;
    let bundled = (url.scheme() == "tauri" && url.host_str() == Some("localhost"))
        || (matches!(url.scheme(), "http" | "https") && url.host_str() == Some("tauri.localhost"));
    let dev = cfg!(debug_assertions)
        && url.scheme() == "http"
        && matches!(url.host_str(), Some("localhost" | "127.0.0.1"))
        && url.port() == Some(1420);
    if window.label() == "main" && (bundled || dev) {
        return Ok(());
    }
    Err(AudioError::new(
        "invalid_window",
        "Audio settings require the local app window.",
    ))
}

#[tauri::command]
pub(crate) fn audio_open_session(
    window: WebviewWindow,
    service: State<'_, AudioService>,
) -> Result<String, AudioError> {
    require_local(&window)?;
    service.open_session()
}

#[tauri::command]
pub(crate) fn audio_close_session(
    window: WebviewWindow,
    service: State<'_, AudioService>,
    session: String,
) -> Result<(), AudioError> {
    require_local(&window)?;
    service.close_session(&session)
}

#[tauri::command]
pub(crate) async fn audio_refresh_devices(
    window: WebviewWindow,
    service: State<'_, AudioService>,
    session: String,
) -> Result<Inventory, AudioError> {
    require_local(&window)?;
    audio::refresh(&service, &session).await
}

#[tauri::command]
pub(crate) fn audio_select_device(
    window: WebviewWindow,
    service: State<'_, AudioService>,
    session: String,
    revision: u32,
    direction: String,
    id: String,
) -> Result<Inventory, AudioError> {
    require_local(&window)?;
    service.select(&session, revision, &direction, &id)
}
