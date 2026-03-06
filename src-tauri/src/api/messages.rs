use crate::prelude;
use crate::net;
                    
use net::{post,post_to_text};

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn sendmessage(
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
pub(crate) async fn getmessages(
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
