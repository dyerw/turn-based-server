use byteorder::{BigEndian, WriteBytesExt};
use bytes::BytesMut;
use nom::{
    character::complete::char,
    multi::length_data,
    number::complete::be_u16,
    sequence::terminated,
    Err::{Error, Failure, Incomplete},
    IResult,
};
use rmp_serde::{from_read, to_vec};
use std::io::Write;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

use crate::messages::Message;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("could not parse netstring")]
    NetstringParseError,
    #[error("could not deserialize msgpack")]
    MsgPackDeserializationError,
    #[error("IOError")]
    Io(#[from] std::io::Error),
}

pub struct MessageCodec;

fn netstring_parser(i: &mut BytesMut) -> IResult<&[u8], &[u8]> {
    terminated(length_data(terminated(be_u16, char(':'))), char(','))(i)
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
    type Item = Message;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Decode netstring
        let parsed = netstring_parser(src);
        match parsed {
            Ok((_remaining, msgpack)) => from_read::<&[u8], Message>(msgpack)
                .map_err(|e| CodecError::MsgPackDeserializationError)
                .map(|f| Some(f)),
            Err(e) => match e {
                Incomplete(_) => Ok(None),
                Failure(er) => Err(CodecError::NetstringParseError),
                Error(er) => Err(CodecError::NetstringParseError),
            },
        }
    }
}

impl Encoder<Message> for MessageCodec {
    type Error = CodecError;
    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let serialize_result = to_vec(&item);

        match serialize_result {
            Ok(msg_pack_buf) => {
                encode_netstring(&msg_pack_buf, dst)?;
                Ok(())
            }
            Err(_) => Err(CodecError::NetstringParseError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{encode_netstring, netstring_parser};
    use bytes::{BufMut, BytesMut};
    use quickcheck_macros::quickcheck;

    #[test]
    fn test_netstring_parser() {
        let ns: &[u8] = &[
            0x00, 0x0c, 0x3a, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
            0x21, 0x2c,
        ];
        let bytes = &mut BytesMut::with_capacity(16);
        bytes.put(ns);

        let result = netstring_parser(bytes);

        let expected: (&[u8], &[u8]) = (
            &[],
            &[
                0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x21,
            ],
        );
        assert_eq!(Ok(expected), result)
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
            Ok((_rem, item2)) => item == item2,
            Err(_) => false,
        }
    }
}
