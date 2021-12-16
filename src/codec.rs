use byteorder::{BigEndian, WriteBytesExt};
use bytes::{Buf, BytesMut};
use nom::{
    character::complete::char, multi::length_data, number::complete::be_u16, sequence::terminated,
    Err::Incomplete, IResult,
};
use rmp_serde::{from_read, Serializer};
use serde::Serialize;
use std::io::Write;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

use crate::messages::NetworkMessage;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("could not parse netstring")]
    NetstringParseError,
    #[error("could not deserialize msgpack")]
    MsgPackDeserializationError,
    #[error("could not serialize msgpack for message {0:?}")]
    MsgPackSerializationError(NetworkMessage),
    #[error("IOError")]
    Io(#[from] std::io::Error),
}

pub struct MessageCodec;

fn netstring_parser(i: &mut BytesMut) -> Result<Option<Vec<u8>>, CodecError> {
    let cpy = &i.clone();
    let parsed: IResult<&[u8], &[u8]> =
        terminated(length_data(terminated(be_u16, char(':'))), char(','))(cpy);

    match parsed {
        Ok((remaining, value)) => {
            i.advance(i.len() - remaining.len());
            Ok(Some(value.into()))
        }
        Err(Incomplete(_)) => Ok(None),
        e => {
            println!("{:?}", e);
            Err(CodecError::NetstringParseError)
        }
    }
}

fn encode_netstring(item: &Vec<u8>, dst: &mut BytesMut) -> Result<(), CodecError> {
    let item_len = item.len() as u16;
    let mut bytes: Vec<u8> = Vec::new();
    bytes.write_u16::<BigEndian>(item_len)?;
    bytes.write_u8(b':')?;
    bytes.write(item)?;
    bytes.write_u8(b',')?;
    dst.extend(bytes);
    Ok(())
}

impl Decoder for MessageCodec {
    type Item = NetworkMessage;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 {
            return Ok(None);
        }

        let parsed = netstring_parser(src);
        match parsed {
            Ok(Some(msgpack)) => from_read::<&[u8], NetworkMessage>(msgpack.as_slice())
                .map_err(|e| CodecError::MsgPackDeserializationError)
                .map(|f| Some(f)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl Encoder<NetworkMessage> for MessageCodec {
    type Error = CodecError;
    fn encode(&mut self, item: NetworkMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut serialized = Vec::new();
        item.serialize(
            &mut Serializer::new(&mut serialized)
                .with_string_variants()
                .with_struct_map(),
        )
        .map_err(|e| CodecError::MsgPackSerializationError(item))?;

        encode_netstring(&serialized, dst)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{encode_netstring, netstring_parser, CodecError, MessageCodec};
    use crate::messages::NetworkMessage;
    use bytes::{BufMut, BytesMut};
    use quickcheck_macros::quickcheck;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_netstring_parser() -> Result<(), CodecError> {
        let ns: &[u8] = &[
            0x00, 0x0c, 0x3a, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
            0x21, 0x2c,
        ];
        let bytes = &mut BytesMut::with_capacity(16);
        bytes.put(ns);

        let result = netstring_parser(bytes)?;

        let expected: Vec<u8> = vec![
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x21,
        ];
        assert_eq!(Some(expected), result);
        Ok(())
    }

    #[test]
    fn test_encode_netstring() {
        let item = vec![
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x21,
        ];
        let dst = &mut BytesMut::with_capacity(16);
        encode_netstring(&item, dst).unwrap();

        let ns: &[u8] = &[
            0x00, 0x0c, 0x3a, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
            0x21, 0x2c,
        ];
        let bytes = &mut BytesMut::with_capacity(16);
        bytes.put(ns);

        assert_eq!(bytes, dst);
    }

    #[quickcheck]
    fn test_netstring_encode_decode_equivalence(item: Vec<u8>) -> bool {
        let dst = &mut BytesMut::with_capacity(100);
        encode_netstring(&item, dst).unwrap();
        let decoded = netstring_parser(dst);
        match decoded {
            Ok(item2) => Some(item) == item2,
            Err(_) => false,
        }
    }

    #[test]
    fn test_encode_decode() -> Result<(), CodecError> {
        let mut codec = MessageCodec {};
        let encoded = &mut BytesMut::with_capacity(0);
        codec.encode(NetworkMessage::CreateLobby { name: "Foo".into() }, encoded)?;

        let decoded = codec.decode(encoded)?;
        assert_eq!(
            Some(NetworkMessage::CreateLobby { name: "Foo".into() }),
            decoded
        );
        assert_eq!(0, encoded.len());

        Ok(())
    }

    #[test]
    fn test_decode_empty() -> Result<(), CodecError> {
        let mut codec = MessageCodec {};
        let encoded = &mut BytesMut::with_capacity(0);

        let decoded = codec.decode(encoded)?;
        assert_eq!(None, decoded);
        assert_eq!(0, encoded.len());

        Ok(())
    }

    #[test]
    fn test_decode_partial() -> Result<(), CodecError> {
        let mut codec = MessageCodec {};
        let encoded = &mut BytesMut::new();
        codec.encode(NetworkMessage::CreateLobby { name: "Foo".into() }, encoded)?;

        encoded.truncate(encoded.len() - 3);
        let prev_len = encoded.len();

        let decoded = codec.decode(encoded)?;

        assert_eq!(prev_len, encoded.len());
        assert_eq!(None, decoded);

        Ok(())
    }

    #[test]
    fn test_decode_whole_and_partial() -> Result<(), CodecError> {
        let mut codec = MessageCodec {};
        let encoded = &mut BytesMut::new();
        codec.encode(NetworkMessage::CreateLobby { name: "Foo".into() }, encoded)?;
        let first_msg_len = encoded.len();
        codec.encode(NetworkMessage::CreateLobby { name: "Bar".into() }, encoded)?;

        encoded.truncate(encoded.len() - 3);
        let prev_len = encoded.len();

        let decoded = codec.decode(encoded)?;

        assert_eq!(prev_len - first_msg_len, encoded.len());
        assert_eq!(
            Some(NetworkMessage::CreateLobby { name: "Foo".into() }),
            decoded
        );

        Ok(())
    }
}
