//! Define who gets sent a particular message. For use in
//! `MessageHandler::handle_client_message`.
//!
//! The [`Recipients`] enum is only used for sending messages through the
//! broadcast channel, and tells the threads for individual connections
//! whether or not they should forward the message to their associated
//! client. For sending a message back to the client whose message you are
//! receiving, use the `channels.response_sender` field.

use serde::{Deserialize, Serialize};

/// Enum representing who the server should send a given message to.
/// The type parameter `T` should be the type used for client IDs.
///
/// Sending a message with recipients [`Recipients::SingleRecipient`] will only
/// send it to the client whose assigned ID matches the internal client ID.
///
/// Sending a message with recipients [`Recipients::MultipleRecipients`] will
/// forward the message to all clients whose ID matches one in the recipients
/// list.
///
/// Sending with recipients [`Recipients::Everyone`] will forward it to all
/// clients.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Recipients<T> {
    /// For sending to a single other client.
    SingleRecipient {
        /// The client ID to send the message to.
        recipient: T,
    },
    /// For sending to multiple other clients.
    MultipleRecipients {
        /// The list of client IDs to send the message to.
        recipients: Vec<T>,
    },
    /// For sending to all clients.
    Everyone,
}

impl<T: PartialEq> Recipients<T> {
    /// Creates a [`Recipients`] object representing all except one of the clients.
    /// To use this function, `T` must implement [`PartialEq`].
    pub fn everyone_but(client_id: &T, clients: impl IntoIterator<Item = T>) -> Recipients<T> {
        Recipients::MultipleRecipients {
            recipients: clients.into_iter().filter(|x| x != client_id).collect(),
        }
    }
}
