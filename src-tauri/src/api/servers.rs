use crate::prelude;
use crate::net;
                    
use net::{post_to_text, get_to_text};

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn getservers(token: String, username: String) -> String {
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
pub(crate) async fn get_server_info(server_id: String) -> String {
    let res = get_to_text(&format!("https://onlinedi.vision/servers/{}/api/get_server_info",server_id), "Failed to get server info.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn create_server(
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
pub(crate) async fn join_server(
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
pub(crate) async fn get_server_users(
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

