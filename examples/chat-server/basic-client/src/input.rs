use async_trait::async_trait;

use chat_api::api::ClientMessage;
use futures::SinkExt;
use scot::{client::InputHandler, types::ValueSender};

pub struct Inputs;

#[async_trait]
impl InputHandler for Inputs {
    async fn next_input(message_channel: &mut ValueSender) {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Error while reading user input!");
        let trimmed = input.trim_matches(char::is_whitespace);
        match trimmed {
            "" => {}
            "/ping" => {
                message_channel
                    .send(serde_json::to_value(&ClientMessage::Ping).unwrap())
                    .await
                    .unwrap();
            }
            _ => {
                message_channel
                    .send(
                        serde_json::to_value(&ClientMessage::ChatMessage {
                            message: trimmed.to_string(),
                        })
                        .unwrap(),
                    )
                    .await
                    .unwrap();
            }
        }
    }
}
