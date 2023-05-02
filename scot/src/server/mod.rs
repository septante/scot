//! A collection of traits for servers and their data types to implement.
//!
//! Creating a server consists of the following steps:
//! - Defining an API consisting of a server message type and a client message type
//! - Defining a type to use for client IDs
//! - Defining a [`State`] type
//! - Defining a [`MessageHandler`]
//! - Defining a [`Server`] struct
//! - Starting the server

mod state;

pub mod recipients;

pub use recipients::Recipients;
pub use state::State;

use anyhow::{Error, Result};
use async_trait::async_trait;
use futures::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
};
use tokio_serde::formats::SymmetricalJson;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::types::*;

/// Trait representing a server object.
///
/// Associated types
/// - `State`: the type used for storing server state
/// - `ClientID`: the type used for IDs
/// - `ClientMessage`: the client message type from the API
/// - `ClientMessageHandler`: a Handler for ClientMessages
///
/// ```no_run
/// use async_trait::async_trait;
/// # use scot::Server;
/// # use scot::server::MessageHandler;
/// # use scot::server::State;
/// # use scot::types::ServerMessageChannels;
/// # use scot::types::ValueSender;
/// # use serde::{Serialize, Deserialize};
/// #
/// # pub struct ServerState;
/// # impl State for ServerState {
/// #     type ClientID = usize;
/// #     fn on_join(&mut self) -> Self::ClientID {
/// #         todo!();
/// #     }
/// # }
/// #
/// # #[derive(Serialize, Deserialize)]
/// # struct ChatClientMessage;
/// # struct ClientMessageHandler;
/// # #[async_trait]
/// # impl MessageHandler for ClientMessageHandler {
/// #     type ClientMessage = ChatClientMessage;
/// #     type ClientID = usize;
/// #     type State = ServerState;
/// #
/// #     async fn handle_client_message(
/// #         msg: ChatClientMessage,
/// #         id: &usize,
/// #         channels: &mut ServerMessageChannels<usize>,
/// #         state: &mut ServerState
/// #     ) {}
/// # }
///
/// struct ChatServer {
///     state: ServerState
/// }
///
/// #[async_trait]
/// impl Server for ChatServer {
///     type State = ServerState;
///     type ClientID = usize;
///     type ClientMessage = ChatClientMessage;
///     type ClientMessageHandler = ClientMessageHandler;
///
///     fn get_state(&self) -> ServerState {
///         todo!();
///     }
/// }
///
/// #[tokio::main]
/// pub async fn main() {
///     let server = ChatServer{ state: ServerState{} };
///     server.start("localhost:1234").await;
/// }
/// ```
#[async_trait]
pub trait Server: 'static {
    /// A type representing the server state. Must implement [`State`].
    type State: State<ClientID = Self::ClientID> + Send;
    /// The type to use for client IDs. Suggested types: `Uuid` or [`usize`].
    type ClientID: 'static + Clone + Serialize + DeserializeOwned + PartialEq + Send + Sync;
    /// The messages to be received from the client. Should be defined in your server API.
    /// Will often be an enum.
    type ClientMessage: 'static + Serialize + DeserializeOwned + Unpin + Send;
    /// A type that implements [`MessageHandler`] for the given client message
    /// and ID types.
    type ClientMessageHandler: MessageHandler<
            ClientMessage = Self::ClientMessage,
            ClientID = Self::ClientID,
            State = Self::State,
        > + Send;

    /// Get a copy of the [`State`].
    fn get_state(&self) -> Self::State;

    /// Start the server on the given address.
    async fn start(&self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        self.start_with_listener(&listener).await
    }

    /// Start the server with a [`TcpListener`].
    async fn start_with_listener(&self, listener: &TcpListener) -> Result<()> {
        let (broadcast_sender, _rx) = broadcast::channel::<(Value, Recipients<Self::ClientID>)>(10);

        loop {
            self.__next_client(listener, &broadcast_sender, self.get_state())
                .await?;
        }
    }

    #[doc(hidden)]
    /// Accept the next connection and set up channels.
    async fn __next_client(
        &self,
        listener: &TcpListener,
        broadcast_sender: &BroadcastSender<Self::ClientID>,
        mut state: Self::State,
    ) -> Result<()> {
        let (stream, _addr) = listener.accept().await?;

        let broadcast_sender = broadcast_sender.clone();
        let mut broadcast_receiver: BroadcastReceiver<Self::ClientID> =
            broadcast_sender.subscribe();

        let id: Self::ClientID = state.on_join();

        // Duplicate the socket: one for serializing and one for deserializing
        let de_stream = stream.into_std()?;
        let ser_stream = de_stream.try_clone()?;
        let de_stream = TcpStream::from_std(de_stream)?;
        let ser_stream = TcpStream::from_std(ser_stream)?;

        let mut client_message_receiver: MessageReceiver<Self::ClientMessage> =
            tokio_serde::SymmetricallyFramed::new(
                FramedRead::new(de_stream, LengthDelimitedCodec::new()),
                SymmetricalJson::<Self::ClientMessage>::default(),
            );

        let response_sender: ValueSender = tokio_serde::SymmetricallyFramed::new(
            FramedWrite::new(ser_stream, LengthDelimitedCodec::new()),
            SymmetricalJson::default(),
        );

        // Collect message channels into struct
        let mut message_channels = ServerMessageChannels {
            response_sender,
            broadcast_sender,
        };

        let mut state = self.get_state();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle messages received from the broadcaster and pass them on
                    result = broadcast_receiver.recv() => {
                        match result {
                            Ok((value, recipients)) => {
                                let should_send = match recipients {
                                    Recipients::Everyone => true,
                                    Recipients::SingleRecipient { recipient } => recipient == id,
                                    Recipients::MultipleRecipients { recipients } => {
                                        recipients.contains(&id)
                                    }
                                };

                                if should_send {
                                    let result = message_channels.response_sender.send(value).await;
                                    if let Err(e) = result {
                                        Self::handle_broadcast_send_err(e.into(), &mut state);
                                    }
                                }
                            }
                            Err(e) => {
                                Self::handle_broadcast_recv_err(e.into(), &mut state);
                            }
                        }
                    }

                    // Messages received from the client
                    result = client_message_receiver.try_next() => {
                        match result {
                            Ok(msg) => {
                                if let Some(msg) = msg {
                                    Self::ClientMessageHandler::handle_client_message(msg, &id, &mut message_channels, &mut state).await;
                                }
                            }
                            Err(_e) => {
                                Self::ClientMessageHandler::handle_bad_message(&id, &mut message_channels, &mut state).await;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle broadcast channel send failures.
    ///
    /// Default implementation does nothing.
    fn handle_broadcast_send_err(_err: Error, _state: &mut Self::State) {}

    /// Handle broadcast channel receive failures.
    ///
    /// Default implementation does nothing.
    fn handle_broadcast_recv_err(_err: Error, _state: &mut Self::State) {}
}

/// Trait representing a handler for incoming server messages.
#[async_trait]
pub trait MessageHandler {
    /// The type of incoming server messages.
    /// Should be defined in the server API.
    type ClientMessage;
    /// The type used for client identifiers.
    type ClientID;
    /// The type used by the server to store state.
    type State;

    /// Handle a single incoming client message, optionally modifying the
    /// state and/or sending messages to one or more clients.
    async fn handle_client_message(
        msg: Self::ClientMessage,
        id: &Self::ClientID,
        channels: &mut ServerMessageChannels<Self::ClientID>,
        state: &mut Self::State,
    );

    /// Handle a client message that couldn't be deserialized.
    async fn handle_bad_message(
        _id: &Self::ClientID,
        _channels: &mut ServerMessageChannels<Self::ClientID>,
        _state: &mut Self::State,
    ) {
    }
}
