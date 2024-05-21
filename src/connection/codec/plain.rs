use bytes::{BufMut, Bytes};
use tokio_util::codec::{Decoder, Encoder};
use varuint::*;

use super::{FrameCodec, Message};

pub struct Plain {
  buffer: Vec<u8>,
}

impl Plain {
  pub fn new() -> Self {
    Plain {
      buffer: Vec::new(),
    }
  } 
}

impl FrameCodec for Plain {
  fn parse_frame(&self, src: &mut bytes::BytesMut) -> Result<(u8,u8), std::io::Error> {
    let preamble: u8 = ReadVarint::read_varint(&mut src.as_ref()).unwrap();
    if preamble != 0x00 {
      return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid preamble"));
    }
    let length: u8 = ReadVarint::read_varint(&mut src.as_ref()).unwrap();
    let msg_type: u8 = ReadVarint::read_varint(&mut src.as_ref()).unwrap();

    if src.len() < length as usize {
      return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid message length"));
    }

    Ok((length, msg_type))
  }

  fn get_handshake_frame(&mut self) -> Option<Bytes> {
    None
  }
}

impl Decoder for Plain {
  type Item = Message;
  type Error = std::io::Error;

  fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if src.is_empty() {
      return Ok(None);
    }

    let (length, msg_type) = self.parse_frame(src)?;
    let msg = src.split_to(length as usize);

    Ok(Some(Message {
      protobuf_type: msg_type,
      protobuf_data: msg.to_vec(),
    }))
  }
}

impl Encoder<Message> for Plain {
  type Error = std::io::Error;

  fn encode(&mut self, item: Message, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
    dst.put_u8(0);
    dst.writer().write_varint(item.protobuf_data.len() as u64).unwrap();
    dst.writer().write_varint(item.protobuf_type as u64).unwrap();
    dst.extend_from_slice(&item.protobuf_data);
    Ok(())
  }
}