use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub message: String,
    pub sent_by: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct GetChatDto {
    pub messages: Vec<ChatMessage>,
}
