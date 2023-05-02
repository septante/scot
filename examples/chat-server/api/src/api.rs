use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientMessage {
    Ping,
    ChatMessage { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerMessage {
    PingResponse,
    ChatMessage { user_id: Uuid, message: String },
}
