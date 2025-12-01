use crate::prelude;
use crate::net;
                    
use net::post_to_text;

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn getchannels(host_url: String, token: String, server: String, username: String) -> String {
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

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn create_channel(
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
pub(crate) async fn delete_channel(
	host_url: String,
	username: String,
	token: String,
	server_id: String,
	channel_name: String,
) -> String {
	let mut map = std::collections::HashMap::new();
	map.insert("username", username.clone());
    map.insert("token", token.clone());
    let res = post_to_text(&format!("{}/{}/api/{}/delete_channel",host_url,server_id.clone(),channel_name.clone()), map, "Failed to delete channel.")
        .await
        .expect("err");
    prelude::debug_only_print(&res);
    res
}