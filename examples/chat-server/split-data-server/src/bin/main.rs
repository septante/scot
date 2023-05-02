use anyhow::Result;
use uuid::Uuid;

use scot::Server;

use chat_api::api::ClientMessage;
use split_data_server::state::ServerState;
use split_data_server::ClientMessageHandler;

/// This version of the server locks on fields of the state.
/// See [`ServerState`] for details.
struct ChatServer {
    state: ServerState,
}

impl ChatServer {
    fn new(state: ServerState) -> ChatServer {
        ChatServer { state }
    }
}

impl Server for ChatServer {
    type ClientID = Uuid;
    type ClientMessage = ClientMessage;
    type ClientMessageHandler = ClientMessageHandler;
    type State = ServerState;

    fn get_state(&self) -> ServerState {
        self.state.clone()
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let state = ServerState::default();
    let server: ChatServer = ChatServer::new(state);
    server.start("localhost:31194").await
}
