mod noise;
mod plain;

use std::sync::{Arc, RwLock};

use bytes::Bytes;
pub use noise::Noise;
pub use plain::Plain;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub enum MessageType {
    Request,
    Response,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub message_type: MessageType,
    pub protobuf_type: u32,
    pub protobuf_data: Vec<u8>,
    pub response_type: Option<u32>,
}


pub trait FrameCodec: Encoder<Message, Error = std::io::Error> + Decoder<Item = Message, Error = std::io::Error> {
    fn parse_frame(&self, src: &mut bytes::BytesMut) -> Result<(u8, u8), std::io::Error>;
    fn get_handshake_frame(&mut self) -> Option<Bytes>;
    fn close(&mut self);
}

#[derive(Clone)]
pub enum EspHomeCodec {
    Noise(Arc<RwLock<Noise>>),
    Plain(Arc<RwLock<Plain>>),
}

impl FrameCodec for EspHomeCodec {
    fn parse_frame(&self, src: &mut bytes::BytesMut) -> Result<(u8, u8), std::io::Error> {
        match self {
            EspHomeCodec::Noise(codec) => codec.read().unwrap().parse_frame(src),
            EspHomeCodec::Plain(codec) => codec.read().unwrap().parse_frame(src),
        }
    }

    fn get_handshake_frame(&mut self) -> Option<Bytes> {
        match self {
            EspHomeCodec::Noise(codec) => codec.write().unwrap().get_handshake_frame(),
            EspHomeCodec::Plain(codec) => codec.write().unwrap().get_handshake_frame(),
        }
    }

    fn close(&mut self) {
        match self {
            EspHomeCodec::Noise(codec) => codec.write().unwrap().close(),
            EspHomeCodec::Plain(codec) => codec.write().unwrap().close(),
        }
    }
}

impl Encoder<Message> for EspHomeCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Message, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        match self {
            EspHomeCodec::Noise(codec) => codec.write().unwrap().encode(item, dst),
            EspHomeCodec::Plain(codec) => codec.write().unwrap().encode(item, dst),
        }
    }
}

impl Decoder for EspHomeCodec {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self {
            EspHomeCodec::Noise(codec) => codec.write().unwrap().decode(src),
            EspHomeCodec::Plain(codec) => codec.write().unwrap().decode(src),
        }
    }
}

