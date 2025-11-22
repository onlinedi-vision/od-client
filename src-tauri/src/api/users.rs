use crate::prelude;
use crate::net;
                    
use net::post_to_text;

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn login(password: String, username: String) -> String {
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
pub(crate) async fn signup(password: String, username: String, email: String) -> String {
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
pub(crate) async fn get_profile_pic(
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
pub(crate) async fn set_profile_pic(
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

