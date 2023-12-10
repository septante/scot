use std::sync::{atomic::AtomicUsize, Arc};

use parking_lot::Mutex;
use scot::server::State;
use uuid::Uuid;

/// Adding per-field locks helps reduce time waiting on locks when many
/// operations are attempting to access different fields at the same time.
/// Another possibility is to have a [`Vec<Arc<Mutex<>>>`]. This could be
/// useful if, for instance, you wanted to build a chat server with multiple
/// rooms, with each room being represented by one of the locks in the [`Vec`].
/// Here, we don't want to lock new users out of joining when the message
/// counter is being updated.
#[derive(Default, Clone)]
pub struct ServerState {
    pub users: Arc<Mutex<Vec<Uuid>>>,
    pub message_counter: Arc<AtomicUsize>,
}

impl State for ServerState {
    type ClientID = Uuid;

    fn on_join(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.users.lock().push(id);
        id
    }
}
