use actix::{Actor, Context, Handler, Message, Recipient};

use crate::game::Game;

use super::session::SessionMessage;

pub enum LobbyResponse {
    Ok,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<LobbyResponse, LobbyError>")]
pub enum LobbyMessage {
    JoinLobby(Recipient<SessionMessage>),
}

#[derive(Debug)]
pub struct Lobby {
    game: Option<Game>,
    white_player: Option<Recipient<SessionMessage>>,
    black_player: Option<Recipient<SessionMessage>>,
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

pub enum LobbyError {
    LobbyFull,
    NotEnoughPlayers,
    GameAlreadyStarted,
}

impl Handler<LobbyMessage> for Lobby {
    type Result = Result<LobbyResponse, LobbyError>;

    fn handle(&mut self, msg: LobbyMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            LobbyMessage::JoinLobby(session) => self.add_player(session).map(|_| LobbyResponse::Ok),
        }
    }
}

impl Lobby {
    pub fn new(white_player: Recipient<SessionMessage>) -> Self {
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

    fn add_player(&mut self, player_session: Recipient<SessionMessage>) -> Result<(), LobbyError> {
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
