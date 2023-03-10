use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessageSentEvent {
    pub chat_id: String,
    pub message_id: String,
    pub user_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatCreatedEvent {
    pub chat_id: String,
    pub user_id: String,
    pub subject: String,
}
