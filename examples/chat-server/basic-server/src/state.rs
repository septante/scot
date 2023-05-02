use scot::server::State;
use uuid::Uuid;

#[derive(Default)]
pub struct ServerState {
    pub users: Vec<Uuid>,
    pub message_counter: usize,
}

impl State for ServerState {
    type ClientID = Uuid;

    fn on_join(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.users.push(id);
        id
    }
}
