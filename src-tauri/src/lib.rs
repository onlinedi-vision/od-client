pub(crate) mod prelude;
pub(crate) mod statics;
pub(crate) mod net;
mod api;
                   
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
async fn get_local_token() -> String {
    match std::env::home_dir() {
        Some(home_dih) => {
            std::fs::read_to_string(format!("{}{}", home_dih.display(), get_credential_file()))
                .expect("Should have been able to read the file")
        }
        _ => "Failed".to_string(),
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn write_credentials(creds: String) -> () {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            api::messages::sendmessage,
            api::messages::getmessages,
            
            api::servers::getservers,
            api::servers::get_server_info,
            api::servers::create_server,
            api::servers::join_server,
            api::servers::get_server_users,

            api::channels::getchannels,
            api::channels::create_channel,
            
            api::users::get_profile_pic,
            api::users::set_profile_pic,
            api::users::login,
            api::users::signup,
            
            api::spell_caster::spellcheck,
            
            get_local_token,
            write_credentials,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
