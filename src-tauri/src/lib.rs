use base64::prelude::*;

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
    println!("{}", m_content.clone());
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "{}/{}/api/{}/send_message",
            host_url, server, channel
        ))
        .json(&map)
        .send()
        .await;
    println!("{:?}", res);
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

    let client = reqwest::Client::new();
    let res = client
        .post(
        "https://onlinedi.vision/api/spell/check"
        )
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
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "{}/{}/api/{}/get_messages_migration",
            host_url, server, channel
        ))
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
    let client = reqwest::Client::new();
    let res = client
        .post(

           format!(
            "{}/{}/api/get_server_users",
            host_url, server
        ))
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

#[tauri::command(rename_all = "snake_case")]
async fn getservers(token: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("username", username.clone());
    println!(">>>>> {}", token);
    let client = reqwest::Client::new();
    let res = client
        .post("https://onlinedi.vision/api/get_user_servers")
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

#[tauri::command(rename_all = "snake_case")]
async fn get_server_info(server_id: String) -> String {
    let client = reqwest::Client::new();
    let res = client
        .get(format!(
            "https://onlinedi.vision/servers/{}/api/get_server_info",
            server_id
        ))
        .send()
        .await
        .expect("err")
        .text()
        .await
        .expect("err");
    println!("{:?}", res);
    res
}

#[tauri::command(rename_all = "snake_case")]
async fn getchannels(host_url: String, token: String, server: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("token", token.clone());
    map.insert("username", username.clone());
    println!("getChannels  {}", token);
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/{}/api/get_channels", host_url, server))
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
    println!("here");
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
    println!("write");
    match std::env::home_dir() {
        Some(home_dih) => {
            std::fs::write(
                format!("{}{}", home_dih.display(), get_credential_file()),
                creds,
            )
            .expect("Should have been able to write the file");
        }
        _ => {
            println!("Failed Write");
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn login(password: String, username: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("password", password.clone());
    map.insert("username", username.clone());
    println!("logIn  {}", username);
    let client = reqwest::Client::new();
    let res = client
        .post("https://onlinedi.vision/api/try_login")
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

#[tauri::command(rename_all = "snake_case")]
async fn signup(password: String, username: String, email: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("password", password.clone());
    map.insert("username", username.clone());
    map.insert("email", email.clone());
    println!("signUp  {}", username);
    let client = reqwest::Client::new();
    let res = client
        .post("https://onlinedi.vision/api/new_user")
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

#[tauri::command(rename_all = "snake_case")]
async fn sendfile(cont: String) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert("cont", BASE64_STANDARD.encode(cont.clone()));
    println!("signUp  {}", cont);
    let client = reqwest::Client::new();
    let res = client
        .post("https://onlinedi.vision/api/cdn")
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
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "{}/{}/api/create_channel",
            host_url,
            server_id.clone()
        ))
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
    println!("{:?}", map);
    let client = reqwest::Client::new();
    let res = client
        .post("https://onlinedi.vision/api/create_server")
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
    println!("{:?}", map);
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/{}/api/join", host_url, server_id.clone()))
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

#[tauri::command(rename_all = "snake_case")]
async fn get_profile_pic(
	username: String,
	token: String,
) -> String {
	let mut map = std::collections::HashMap::new();
	map.insert("username", username.clone());
    map.insert("token", token.clone());
	let client = reqwest::Client::new();
	let res = client
		.post("https://onlinedi.vision/api/get_user_pfp")
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
	let client = reqwest::Client::new();
	let res = client
		.post("https://onlinedi.vision/api/set_user_pfp")
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
            sendfile,
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
