mod structures;


#[tauri::command]
fn test() -> (){
    println!("Hello!");
}

#[tauri::command(rename_all="snake_case")]
async fn sendMessage(token: String, channel: String, server: String,  m_content: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("m_content", m_content.clone());
    map.insert("username", username.clone());
    println!("{}", m_content.clone());
    let client = reqwest::Client::new();
    let res = client.post(format!("https://onlinedi.vision/servers/{}/api/{}/send_message", server, channel))
        .json(&map)
        .send()
        .await;
    println!("{:?}", res);
    match res {
        Ok(res) => m_content,
        Err(_) => "err".to_string()
    }
}   

#[tauri::command(rename_all="snake_case")]
async fn getMessages(token: String, channel: String, server: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("username", username.clone());
    let client = reqwest::Client::new();
    let res = client.post(format!("https://onlinedi.vision/servers/{}/api/{}/get_messages", server, channel))
        .json(&map)
        .send()
        .await
        .expect("err")
        .text()
        .await
        .expect("err");
    println!("{:?}", res);
    res
}

#[tauri::command(rename_all="snake_case")]
async fn getServers(token: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("username", username.clone());
    println!(">>>>> {}", token);
    let client = reqwest::Client::new();
    let res = client.post("https://onlinedi.vision/api/get_user_servers")
        .json(&map)
        .send()
        .await
        .expect("err")
        .text()
        .await
        .expect("err");
    println!("{:?}", res);
    res
}

#[tauri::command(rename_all="snake_case")]
async fn getServerInfo(sid: String) -> String {
    let client = reqwest::Client::new();
    let res = client.get(format!("https://onlinedi.vision/servers/{}/api/get_server_info", sid))
        .send()
        .await
        .expect("err")
        .text()
        .await
        .expect("err");
    println!("{:?}", res);
    res
}

#[tauri::command(rename_all="snake_case")]
async fn getChannels(token: String, server: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("username", username.clone());
    println!("getChannels  {}", token);
    let client = reqwest::Client::new();
    let res = client.post(format!("https://onlinedi.vision/servers/{}/api/get_channels", server))
        .json(&map)
        .send()
        .await
        .expect("err")
        .text()
        .await
        .expect("err");
    println!("{:?}", res);
    res
}

#[tauri::command(rename_all="snake_case")]
async fn getLocalToken() -> String {
    println!("here");
    match std::env::home_dir() {
        Some(home_dih) => 
            std::fs::read_to_string(format!("{}/.division-online/credentials.json", home_dih.display()))
                .expect("Should have been able to read the file"),
        _ => "Failed".to_string()
    }
}

#[tauri::command(rename_all="snake_case")]
async fn writeCredentials(creds: String) -> () {
    println!("write");
    match std::env::home_dir() {
        Some(home_dih) => { 
            std::fs::write(format!("{}/.division-online/credentials.json", home_dih.display()), creds)
                .expect("Should have been able to write the file"); 
        },
        _ =>  {
            println!("Failed Write");
        }
    }
}

#[tauri::command(rename_all="snake_case")]
async fn logIn(password: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("password", password.clone());
    map.insert("username", username.clone());
    println!("logIn  {}", username);
    let client = reqwest::Client::new();
    let res = client.post("https://onlinedi.vision/api/try_login")
        .json(&map)
        .send()
        .await
        .expect("err")
        .text()
        .await
        .expect("err");
    println!("{:?}", res);
    res
}

#[tauri::command(rename_all="snake_case")]
async fn signUp(password: String, username: String, email: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("password", password.clone());
    map.insert("username", username.clone());
    map.insert("email", email.clone());
    println!("signUp  {}", username);
    let client = reqwest::Client::new();
    let res = client.post("https://onlinedi.vision/api/new_user")
        .json(&map)
        .send()
        .await
        .expect("err")
        .text()
        .await
        .expect("err");
    println!("{:?}", res);
    res
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            test,
            sendMessage,
            getMessages,
            getChannels,
            getServers,
            getServerInfo,
            getLocalToken,
            writeCredentials,
            logIn,
            signUp
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
