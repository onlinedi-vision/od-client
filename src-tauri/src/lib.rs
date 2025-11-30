pub(crate) mod prelude;
pub(crate) mod statics;
pub(crate) mod net;
mod fs;
mod api;

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
			api::channels::delete_channel,
            
            api::users::get_profile_pic,
            api::users::set_profile_pic,
            api::users::login,
            api::users::signup,
            
            api::spell_caster::spellcheck,
            
            fs::get_local_token,
            fs::write_credentials,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
