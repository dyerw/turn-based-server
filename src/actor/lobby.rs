use actix::{Actor, Addr, Context};

use crate::game::Game;

use super::session::Session;

pub struct Lobby {
    game: Option<Game>,
    white_player: Option<Addr<Session>>,
    black_player: Option<Addr<Session>>,
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

enum LobbyError {
    LobbyFull,
    NotEnoughPlayers,
    GameAlreadyStarted,
}

impl Lobby {
    pub fn new(white_player: Addr<Session>) -> Self {
        Lobby {
            game: None,
            white_player: Some(white_player),
            black_player: None,
        }
    }

    fn start_game(&mut self) -> Result<(), LobbyError> {
        if Option::is_some(&self.white_player) && Option::is_some(&self.black_player) {
            if Option::is_some(&self.game) {
                return Err(LobbyError::GameAlreadyStarted);
            }
            self.game = Some(Game::new());
            Ok(())
        } else {
            Err(LobbyError::NotEnoughPlayers)
        }
    }

    fn add_player(&mut self, player_session: Addr<Session>) -> Result<(), LobbyError> {
        if Option::is_none(&self.white_player) {
            self.white_player = Some(player_session);
            return Ok(());
        }
        if Option::is_none(&self.black_player) {
            self.black_player = Some(player_session);
            return Ok(());
        }
        return Err(LobbyError::LobbyFull);
    }
}
