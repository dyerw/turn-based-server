use serde::{Deserialize, Serialize};

use crate::game::{Color, Position};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "tag", content = "value")]
pub enum NetworkMessage {
    CreateLobby {
        name: String,
    },
    JoinLobby {
        name: String,
    },
    ListLobbiesRequest,
    ListLobbiesResponse {
        lobbies: Vec<String>,
    },
    SetUsername {
        name: String,
    },
    LobbyStateUpdate {
        player1: Option<String>,
        player2: Option<String>,
    },
    MovePiece {
        player: Color,
        from: Position,
        to: Position,
    },
    ServerError(String),
}
