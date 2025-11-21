use crate::prelude;
use crate::net;
                    
use net::post_to_text;

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn spellcheck(
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

