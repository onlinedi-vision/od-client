

#[tauri::command]
fn test() -> (){
    println!("Hello!");
}

#[tauri::command(rename_all="snake_case")]
async fn sendMessage(token: String, m_content: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("m_content", m_content.clone());
    map.insert("username", username.clone());
    println!("{}", m_content.clone());
    let client = reqwest::Client::new();
    let res = client.post("https://onlinedi.vision:1313/servers/1313/api/main/send_message")
        .json(&map)
        .send()
        .await;
    println!("{:?}", res);
    match res {
        Ok(res) => m_content,
        Err(_) => "err".to_string()
    }
}   

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            test,
            sendMessage
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
