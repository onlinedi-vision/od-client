use crate::prelude;

fn get_credential_file() -> String {
    if cfg!(windows) {
        "\\AppData\\Roaming\\OnlineDivision\\credentials.json".to_string()
    } else if cfg!(unix) {
        "/.division-online/credentials.json".to_string()
    } else {
        panic!("FATAL ERROR: unidentified OS type");
    }
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn get_local_token() -> String {
    match std::env::home_dir() {
        Some(home_dih) => {
            std::fs::read_to_string(format!("{}{}", home_dih.display(), get_credential_file()))
                .expect("Should have been able to read the file")
        }
        _ => "Failed".to_string(),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn write_credentials(creds: String) -> () {
    prelude::debug_only_print(&"write");
    match std::env::home_dir() {
        Some(home_dih) => {
            std::fs::write(
                format!("{}{}", home_dih.display(), get_credential_file()),
                creds,
            )
            .expect("Should have been able to write the file");
        }
        _ => {
            prelude::debug_only_print(&"Failed Write");
        }
    }
}

