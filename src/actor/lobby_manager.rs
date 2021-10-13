use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler, Message, MessageResponse, WeakAddr};

use super::{lobby::Lobby, session::Session};

#[derive(Message)]
#[rtype(result = "LobbyManagerResponse")]
pub enum LobbyManagerMessage {
    CreateLobby {
        name: String,
        creating_session: Addr<Session>,
    },
}

#[derive(MessageResponse)]
pub struct LobbyManagerResponse(pub Result<LobbyManagerResponseSuccess, LobbyManagerResponseError>);

pub enum LobbyManagerResponseSuccess {
    CreatedLobby { addr: Addr<Lobby> },
}

pub enum LobbyManagerResponseError {
    LobbyNameTaken,
}

pub struct LobbyManager {
    lobbies: HashMap<String, WeakAddr<Lobby>>,
}

impl Actor for LobbyManager {
    type Context = Context<Self>;
}

impl Handler<LobbyManagerMessage> for LobbyManager {
    type Result = LobbyManagerResponse;
    fn handle(&mut self, msg: LobbyManagerMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            LobbyManagerMessage::CreateLobby {
                name,
                creating_session,
            } => {
                if self.lobbies.contains_key(&name) {
                    LobbyManagerResponse(Err(LobbyManagerResponseError::LobbyNameTaken))
                } else {
                    let lobby = Lobby::new(creating_session);
                    let addr = Lobby::start(lobby);
                    self.lobbies.insert(name, addr.downgrade());
                    LobbyManagerResponse(Ok(LobbyManagerResponseSuccess::CreatedLobby { addr }))
                }
            }
        }
    }
}
