//! Various useful types, mostly relating to sending messages between the
//! server and the client.

use serde_json::Value;
use tokio::{
    net::TcpStream,
    sync::broadcast::{Receiver, Sender},
};
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::server::Recipients;

pub(crate) type BroadcastSender<T> = Sender<(Value, Recipients<T>)>;
pub(crate) type BroadcastReceiver<T> = Receiver<(Value, Recipients<T>)>;

pub(crate) type MessageReceiver<T> =
    Framed<FramedRead<TcpStream, LengthDelimitedCodec>, T, T, Json<T, T>>;
pub(crate) type MessageSender<T> =
    Framed<FramedWrite<TcpStream, LengthDelimitedCodec>, T, T, Json<T, T>>;

/// A channel that can be used to send serde JSON values.
///
/// This mainly shows up in internal code, but is also used in
/// [`crate::client::InputHandler`] as the type of the channel
/// through which the client can send messages to the server.
pub type ValueSender = MessageSender<Value>;

/// Channels the server can use to send messages to clients.
/// broadcast_sender is for sending to multiple clients, while
/// value_sender is for sending messages back to the specific client
/// attached to value_sender.
#[non_exhaustive]
pub struct ServerMessageChannels<T> {
    /// Channel for sending messages back to the associated client.
    pub response_sender: ValueSender,
    /// Channel to be used for sending messages across threads,
    /// i.e., for sending to other clients.
    pub broadcast_sender: BroadcastSender<T>,
}
