pub(crate) mod prelude;
pub(crate) mod statics;
pub(crate) mod net;
                    
use net::{post,post_to_text, get_to_text};

#[tauri::command(rename_all = "snake_case")]
async fn sendmessage(
    host_url: String,
    token: String,
    channel: String,
    server: String,
    m_content: String,
    username: String,
) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("m_content", m_content.clone());
    map.insert("username", username.clone());
    prelude::debug_only_print(&m_content.clone());
    let res = post(&format!("{}/{}/api/{}/send_message",host_url, server, channel), map).await;
    prelude::debug_only_print(&res);
    match res {
        Ok(_) => m_content,
        Err(_) => "err".to_string(),
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn spellcheck(
    token: String,
    username: String,
    key: String,
) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("username", username.clone());
    map.insert("key", key.clone());

    let res = post_to_text("https://onlinedi.vision/api/spell/check", map, "Spell Check Failed").await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn getmessages(
    host_url: String,
    token: String,
    channel: String,
    server: String,
    username: String,
) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("username", username.clone());
    map.insert("limit", "100".to_string());
    map.insert("offset", "0".to_string());
    let res = post_to_text(&format!("{}/{}/api/{}/get_messages_migration",host_url, server, channel), map, "Failed to retrieve messages")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}


#[tauri::command(rename_all = "snake_case")]
async fn get_server_users(
    host_url: String,
    token: String,
    server: String,
    username: String
) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("username", username.clone());
    let res = post_to_text(&format!("{}/{}/api/get_server_users",host_url, server), map, "Failed to get server users.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn getservers(token: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("username", username.clone());
    prelude::debug_only_print(&format!("getservers {}", token));
    let res = post_to_text("https://onlinedi.vision/api/get_user_servers", map, "Failed to get use servers.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn get_server_info(server_id: String) -> String {
    let res = get_to_text(&format!("https://onlinedi.vision/servers/{}/api/get_server_info",server_id), "Failed to get server info.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn getchannels(host_url: String, token: String, server: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("username", username.clone());
    prelude::debug_only_print(&token);
    let res = post_to_text(&format!("{}/{}/api/get_channels", host_url, server), map, "Failed to get channels.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

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

#[tauri::command(rename_all = "snake_case")]
async fn login(password: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("password", password.clone());
    map.insert("username", username.clone());
    prelude::debug_only_print(&username);
    let res = post_to_text("https://onlinedi.vision/api/try_login", map, "Failed to Log in.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn signup(password: String, username: String, email: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("password", password.clone());
    map.insert("username", username.clone());
    map.insert("email", email.clone());
    prelude::debug_only_print(&username);
    let res = post_to_text("https://onlinedi.vision/api/new_user", map, "Failed to create new user.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn create_channel(
    host_url: String,
    username: String,
    token: String,
    server_id: String,
    channel_name: String,
) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("username", username.clone());
    map.insert("channel_name", channel_name.clone());
    map.insert("token", token.clone());
    let res = post_to_text(&format!("{}/{}/api/create_channel",host_url,server_id.clone()), map, "Failed to create channel.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn create_server(
    username: String,
    token: String,
    name: String,
    img_url: String,
    desc: String,
) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("username", username.clone());
    map.insert("name", name.clone());
    map.insert("desc", desc.clone());
    map.insert("img_url", img_url.clone());
    map.insert("token", token.clone());
    prelude::debug_only_print(&map);
    let res = post_to_text("https://onlinedi.vision/api/create_server", map, "Failed to create server.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn join_server(
    host_url: String,
    username: String,
    token: String,
    server_id: String,
) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("username", username.clone());
    map.insert("token", token.clone());
    prelude::debug_only_print(&map);
    let res = post_to_text(&format!("{}/{}/api/join", host_url, server_id.clone()), map, "Failed to join server.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn get_profile_pic(
	username: String,
	token: String,
) -> String {
	let mut map = std::collections::HashMap::new();
	map.insert("username", username.clone());
    map.insert("token", token.clone());
	let res = post_to_text("https://onlinedi.vision/api/get_user_pfp", map, "Failed to get PFP.")
		.await
		.expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn set_profile_pic(
	username: String,
	token: String,
	img_url: String,
) -> String {
	let mut map = std::collections::HashMap::new();
	map.insert("username", username.clone());
    map.insert("token", token.clone());
	map.insert("img_url", img_url.clone());
	let res = post_to_text("https://onlinedi.vision/api/set_user_pfp", map, "Failed to set profile pic.")
		.await
		.expect("err");
    prelude::debug_only_print(&res);
    res
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            sendmessage,
            getmessages,
            getchannels,
            getservers,
            get_server_info,
            get_local_token,
            write_credentials,
            login,
            signup,
            create_channel,
            create_server,
            join_server,
            get_server_users,
            spellcheck,
            get_profile_pic,
            set_profile_pic,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
