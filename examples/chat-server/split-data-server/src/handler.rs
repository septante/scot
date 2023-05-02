use async_trait::async_trait;
use futures::SinkExt;

use crate::state::ServerState;
use chat_api::api::{ClientMessage, ServerMessage};
use scot::{server::recipients::Recipients, server::MessageHandler, types::*};
use uuid::Uuid;

#[derive(Clone)]
pub struct ClientMessageHandler;

#[async_trait]
#[allow(unreachable_patterns)]
impl MessageHandler for ClientMessageHandler {
    type ClientMessage = ClientMessage;
    type ClientID = Uuid;
    type State = ServerState;

    async fn handle_client_message(
        msg: ClientMessage,
        user_id: &Uuid,
        message_channels: &mut ServerMessageChannels<Uuid>,
        state: &mut ServerState,
    ) {
        match msg {
            ClientMessage::Ping => {
                println!("Got a ping from user {}!", user_id);
                message_channels
                    .response_sender
                    .send(serde_json::to_value(ServerMessage::PingResponse).unwrap())
                    .await
                    .unwrap();
            }
            ClientMessage::ChatMessage { message } => {
                let message = serde_json::to_value(ServerMessage::ChatMessage {
                    user_id: *user_id,
                    message,
                })
                .unwrap();
                let users: Vec<Uuid> = { state.users.lock().clone() };
                let recipients = Recipients::everyone_but(user_id, users);

                {
                    *state.message_counter.lock() += 1;
                    println!("Total messages: {}", state.message_counter.lock());
                }

                message_channels
                    .broadcast_sender
                    .send((message, recipients))
                    .unwrap();
            }

            _ => {
                println!("Got a message from the client that couldn't be understood")
            }
        }
    }
}
