use std::sync::Arc;

use parking_lot::Mutex;
use scot::server::State;
use uuid::Uuid;

/// Adding per-field locks helps reduce time waiting on locks when many
/// operations are attempting to access different fields at the same time.
/// Another possibility is to have a [`Vec<Arc<Mutex<>>>`]. This could be
/// useful if, for instance, you wanted to build a chat server with multiple
/// rooms, with each room being represented by one of the locks in the [`Vec`].
#[derive(Default, Clone)]
pub struct ServerState {
    pub users: Arc<Mutex<Vec<Uuid>>>,
    pub message_counter: Arc<Mutex<usize>>,
}

impl State for ServerState {
    type ClientID = Uuid;

    fn on_join(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.users.lock().push(id);
        id
    }
}
