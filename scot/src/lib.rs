//! scot: Server-Client over TCP.

#![forbid(unsafe_code)]
#[warn(clippy::pedantic)]
#[warn(missing_docs)]
pub mod client;
pub mod server;
pub mod types;

pub use client::Client;
pub use server::Server;

/// Trait and marker to prevent external users from calling trait functions
mod private {
    pub trait Internal {}

    pub enum InternalFlag {}
    impl Internal for InternalFlag {}
}
