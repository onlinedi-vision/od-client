#[derive(serde::Deserialize)]
pub struct MessageList {
    pub m_list: Vec<Message>,
}

#[derive(serde::Deserialize)]
pub struct Message {
    pub m_content: String,
    pub username: String,
    pub datetime: String,
}
