use bytes::{Buf, BytesMut};

#[derive(Debug, PartialEq)]
pub enum LobbyMessage {
    CreateLobby { name: String },
}

impl LobbyMessage {
    pub fn decode(src: &mut BytesMut) -> Result<Option<Self>, LobbyMessageError> {
        match src[0] {
            0x10 => {
                let name_len = src[1];
                if src.len() < (name_len + 2).into() {
                    return Ok(None);
                }

                src.advance(2);

                String::from_utf8(src.split_to(name_len.into()).to_vec())
                    .map(|name| Some(LobbyMessage::CreateLobby { name }))
                    .map_err(|_| LobbyMessageError::LobbyNameUtf8Error)
            }
            _ => Err(LobbyMessageError::CannotDecodeNonLobbyMessage),
        }
    }
    pub fn encode(item: Self, dst: &mut BytesMut) -> Result<(), LobbyMessageError> {
        match item {
            LobbyMessage::CreateLobby { name } => {
                if name.len() > 255 {
                    return Err(LobbyMessageError::LobbyNameTooLong);
                }
                let mut frame: Vec<u8> = vec![0x10u8, name.len() as u8];
                frame.extend(name.into_bytes());
                dst.extend(frame);
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LobbyMessageError {
    LobbyNameTooLong,
    LobbyNameUtf8Error,
    CannotDecodeNonLobbyMessage,
}

#[cfg(test)]
mod tests {

    use super::{LobbyMessage, LobbyMessageError};
    use bytes::BytesMut;

    #[test]
    fn test_create_lobby_codec() {
        let create_lobby = LobbyMessage::CreateLobby {
            name: "My Lobby".into(),
        };

        let bytes = &mut BytesMut::new();
        let encoded_frame = LobbyMessage::encode(create_lobby, bytes);

        assert_eq!(encoded_frame, Ok(()));

        let decoded_frame = LobbyMessage::decode(bytes);

        assert_eq!(
            Ok(Some(LobbyMessage::CreateLobby {
                name: "My Lobby".into(),
            })),
            decoded_frame
        );
    }

    #[test]
    fn test_create_lobby_too_long() {
        let create_lobby = LobbyMessage::CreateLobby {
            name: ['a'; 256].iter().cloned().collect(),
        };

        let bytes = &mut BytesMut::new();
        let encoded_frame = LobbyMessage::encode(create_lobby, bytes);
        assert_eq!(encoded_frame, Err(LobbyMessageError::LobbyNameTooLong))
    }
}
