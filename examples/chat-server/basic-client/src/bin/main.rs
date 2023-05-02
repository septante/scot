use anyhow::Result;
use basic_chat_client::{handler::ServerMessageHandler, input::Inputs};
use chat_api::api::ServerMessage;
use scot::Client;

struct ChatClient;

impl Client for ChatClient {
    type ServerMessage = ServerMessage;
    type ServerMessageHandler = ServerMessageHandler;
    type InputHandler = Inputs;
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = ChatClient {};
    client.start("localhost:31194").await
}
