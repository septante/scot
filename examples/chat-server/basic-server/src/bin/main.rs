use std::sync::Arc;

use anyhow::Result;
use parking_lot::Mutex;
use uuid::Uuid;

use scot::Server;

use basic_chat_server::state::ServerState;
use basic_chat_server::ClientMessageHandler;
use chat_api::api::ClientMessage;

/// This server uses the simplest way to share data, which is to wrap
/// the entire state in an [`Arc<Mutex<State>>`].
/// Compare this with `split-data-server`, which has separate
/// [`Arc<Mutex<_>>`]s in each field of the state.
struct ChatServer {
    state: Arc<Mutex<ServerState>>,
}

impl ChatServer {
    fn new(state: ServerState) -> ChatServer {
        ChatServer {
            state: Arc::new(Mutex::new(state)),
        }
    }
}

impl Server for ChatServer {
    type ClientID = Uuid;
    type ClientMessage = ClientMessage;
    type ClientMessageHandler = ClientMessageHandler;
    type State = Arc<Mutex<ServerState>>;

    fn get_state(&self) -> Arc<Mutex<ServerState>> {
        self.state.clone()
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let state = ServerState::default();
    let server: ChatServer = ChatServer::new(state);
    server.start("localhost:31194").await
}
