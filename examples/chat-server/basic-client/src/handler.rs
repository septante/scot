use async_trait::async_trait;
use chat_api::api::ServerMessage;
use scot::client::MessageHandler;

#[derive(Clone)]
pub struct ServerMessageHandler;

#[async_trait]
#[allow(unreachable_patterns)]
impl MessageHandler for ServerMessageHandler {
    type ServerMessage = ServerMessage;

    async fn handle_server_message(msg: ServerMessage) {
        match msg {
            ServerMessage::PingResponse => {
                println!("pong!");
            }
            ServerMessage::ChatMessage { user_id, message } => {
                println!("User #{}: {}", user_id, message);
            }
            _ => {
                println!("Got a message from the server that the client couldn't understand!")
            }
        }
    }
}
