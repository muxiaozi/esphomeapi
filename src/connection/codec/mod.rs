mod noise;
mod plain;

use bytes::Bytes;
pub use noise::Noise;
pub use plain::Plain;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub struct Message {
    protobuf_type: u8,
    protobuf_data: Vec<u8>,
}

pub trait FrameCodec {
    fn parse_frame(&self, src: &mut bytes::BytesMut) -> Result<(u8, u8), std::io::Error>;
    fn get_handshake_frame(&mut self) -> Option<Bytes>;
}

pub enum EspHomeCodec {
    Noise(Noise),
    Plain(Plain),
}

impl FrameCodec for EspHomeCodec {
    fn parse_frame(&self, src: &mut bytes::BytesMut) -> Result<(u8, u8), std::io::Error> {
        match self {
            EspHomeCodec::Noise(codec) => codec.parse_frame(src),
            EspHomeCodec::Plain(codec) => codec.parse_frame(src),
        }
    }

    fn get_handshake_frame(&mut self) -> Option<Bytes> {
        match self {
            EspHomeCodec::Noise(codec) => codec.get_handshake_frame(),
            EspHomeCodec::Plain(codec) => codec.get_handshake_frame(),
        }
    }
}

impl Encoder<Message> for EspHomeCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Message, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        match self {
            EspHomeCodec::Noise(codec) => codec.encode(item, dst),
            EspHomeCodec::Plain(codec) => codec.encode(item, dst),
        }
    }
}

impl Decoder for EspHomeCodec {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self {
            EspHomeCodec::Noise(codec) => codec.decode(src),
            EspHomeCodec::Plain(codec) => codec.decode(src),
        }
    }
}

