//! A collection of traits for clients and their data types to implement
//!
//! Creating a client consists of the following steps:
//! - Importing the API that the server is using
//! - Defining a [`MessageHandler`] to handle incoming server messages
//! - Defining an [`InputHandler`] to receive input from the client and
//! respond appropriately, sending messages to the server when needed
//! - Defining a [`Client`] struct
//! - Starting the client

use crate::types::{MessageReceiver, ValueSender};

use anyhow::{Error, Result};
use async_trait::async_trait;
use futures::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;
use tokio_serde::formats::SymmetricalJson;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

/// The base trait for the client half of the client-server
///
/// To use, create a struct with an `impl Client` block and declare the relevant types,
/// then instantiate a copy of the struct and call its `.start()` method.
///
/// ```no_run
/// use async_trait::async_trait;
/// # use serde::{Serialize, Deserialize};
/// # use scot::Client;
/// # use scot::client::{MessageHandler, InputHandler};
/// # use scot::Server;
/// # use scot::types::ValueSender;
/// #
/// # #[derive(Serialize, Deserialize)]
/// # struct ChatServerMessage;
/// # struct ServerMessageHandler;
/// # #[async_trait]
/// # impl MessageHandler for ServerMessageHandler {
/// #     type ServerMessage = ChatServerMessage;
/// #     async fn handle_server_message(msg: ChatServerMessage) {}
/// # }
/// # struct GUIInputHandler;
/// #
/// # #[async_trait]
/// # impl InputHandler for GUIInputHandler {
/// #   async fn next_input(serialized: &mut ValueSender) {}
/// # }
///
/// struct ChatClient;
///
/// #[async_trait]
/// impl Client for ChatClient {
///     type ServerMessage = ChatServerMessage;
///     type ServerMessageHandler = ServerMessageHandler;
///     type InputHandler = GUIInputHandler;
/// }
///
/// #[tokio::main]
/// pub async fn main() {
///     let client = ChatClient{};
///     client.start("localhost:1234").await;
/// }
/// ```
#[async_trait]
pub trait Client {
    /// The type representing messages received from the server. Should be
    /// imported from the server's API.
    type ServerMessage: 'static + Serialize + DeserializeOwned + Unpin + Send;
    /// A type implementing [`MessageHandler`] for the given [`Self::ServerMessage`] type
    type ServerMessageHandler: MessageHandler<ServerMessage = Self::ServerMessage>;
    /// Implements [`InputHandler`], which accepts input from the client in
    /// some form and responds, possibly sending messages to the server.
    type InputHandler: InputHandler;

    /// Start the client and connect to the given address.
    async fn start(&self, addr: &str) -> Result<()> {
        let stream = TcpStream::connect(addr).await?;
        self.start_with_stream(stream).await
    }

    /// Start the client with a given TcpStream.
    async fn start_with_stream(&self, stream: TcpStream) -> Result<()> {
        // Duplicate the stream: one for serializing and one for deserializing
        let de_stream = stream.into_std()?;
        let ser_stream = de_stream.try_clone()?;
        let de_stream = TcpStream::from_std(de_stream)?;
        let ser_stream = TcpStream::from_std(ser_stream)?;

        let mut deserialized: MessageReceiver<Self::ServerMessage> =
            tokio_serde::SymmetricallyFramed::new(
                FramedRead::new(de_stream, LengthDelimitedCodec::new()),
                SymmetricalJson::<Self::ServerMessage>::default(),
            );

        let mut serialized: ValueSender = tokio_serde::SymmetricallyFramed::new(
            FramedWrite::new(ser_stream, LengthDelimitedCodec::new()),
            SymmetricalJson::default(),
        );

        // Handle incoming messages from the server
        tokio::spawn(async move {
            while let Some(next) = deserialized.next().await {
                match next {
                    Ok(msg) => Self::ServerMessageHandler::handle_server_message(msg).await,
                    Err(e) => Self::ServerMessageHandler::handle_bad_message(e.into()).await,
                }
            }
        });

        // Continuously read user input and send appropriate messages to the server
        loop {
            Self::InputHandler::next_input(&mut serialized).await;
        }
    }
}

/// Trait representing a handler for incoming server messages.
#[async_trait]
pub trait MessageHandler {
    /// Type representing messages received from the server. Should be
    /// imported from the server API.
    type ServerMessage;

    /// Do something with the given server message.
    async fn handle_server_message(msg: Self::ServerMessage);
    /// Handle server messages whose deserialization fails.
    ///
    /// Default implementation does nothing.
    async fn handle_bad_message(_err: Error) {}
}

/// A trait for accepting user input.
#[async_trait]
pub trait InputHandler {
    /// Get input from the client and optionally send a message to the server
    /// using the given channel.
    async fn next_input(message_channel: &mut ValueSender);
}
