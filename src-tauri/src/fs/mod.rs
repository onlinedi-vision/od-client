use tauri::Manager;

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn get_local_token(app: tauri::AppHandle) -> String {
    let path = app.path().app_data_dir().unwrap().join("credentials.json");
    std::fs::read_to_string(&path).unwrap_or_else(|_| "Failed".to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn write_credentials(app: tauri::AppHandle, creds: String) {
    let dir = app.path().app_data_dir().unwrap();
    std::fs::create_dir_all(&dir).expect("Should have been able to create app data directory");
    let path = dir.join("credentials.json");
    #[cfg(debug_assertions)]
    println!("write: {}", path.display());
    std::fs::write(&path, creds).expect("Should have been able to write the file");
}
