use serde::{Deserialize, Serialize};

pub struct Undefined(());

#[derive(Serialize, Deserialize, Clone)]
pub struct JsonResponse<T: Serialize> {
    pub data: T,
    pub message: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ChatMessage {
    pub message: String,
    pub sent_by: String,
    pub message_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetChatDto {
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateChatDto {
    pub user_id: String,
    pub chat_id: String,
}
