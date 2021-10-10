use std::sync::{Arc, Mutex};

use crate::game::Game;

#[derive(Clone)]
pub struct ServerState {
    pub game: Arc<Mutex<Game>>,
}

impl ServerState {
    pub fn new() -> ServerState {
        ServerState {
            game: Arc::new(Mutex::new(Game::new())),
        }
    }
}
