use serde::{Deserialize, Serialize};

use crate::game::{Color, Position};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "tag", content = "value")]
pub enum Message {
    CreateLobby {
        name: String,
    },
    JoinLobby {
        name: String,
    },
    MovePiece {
        player: Color,
        from: Position,
        to: Position,
    },
}
