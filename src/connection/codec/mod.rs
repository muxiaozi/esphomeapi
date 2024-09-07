mod noise;
mod plain;

use std::sync::{Arc, RwLock};

use bytes::Bytes;
pub use noise::Noise;
pub use plain::Plain;
use tokio::sync::oneshot;
use tokio_util::codec::{Decoder, Encoder};

pub type DynamicMessage = Box<dyn protobuf::MessageDyn>;
pub type DynamicResponseMessage = (u32, DynamicMessage);

#[derive(Debug, Clone)]
// A common struct that holds shared fields between messages
pub struct ProtobufMessage {
    pub protobuf_type: u32,
    pub protobuf_data: Vec<u8>,
}

pub enum MessageType {
    Response {
        protobuf_message: ProtobufMessage,
    },
    Request {
        protobuf_message: ProtobufMessage,
    },
    RequestWithAwait {
        protobuf_message: ProtobufMessage,
        response_protobuf_type: u32,
    },
    RequestWithAwaitFn {
        protobuf_message: ProtobufMessage,
        response_protobuf_type: u32,
        callback: Box<dyn FnOnce(ProtobufMessage) + Send + 'static>,
    },
}

impl std::fmt::Debug for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Response { protobuf_message } => {
                f.debug_struct("Response")
                    .field("protobuf_type", &protobuf_message.protobuf_type)
                    .field("protobuf_data", &protobuf_message.protobuf_data)
                    .finish()
            }
            MessageType::Request { protobuf_message } => {
                f.debug_struct("Request")
                    .field("protobuf_type", &protobuf_message.protobuf_type)
                    .field("protobuf_data", &protobuf_message.protobuf_data)
                    .finish()
            }
            MessageType::RequestWithAwait { protobuf_message, .. } => {
                f.debug_struct("RequestWithAwait")
                    .field("protobuf_type", &protobuf_message.protobuf_type)
                    .field("protobuf_data", &protobuf_message.protobuf_data)
                    .finish()
            }
            MessageType::RequestWithAwaitFn { protobuf_message, .. } => {
                f.debug_struct("RequestWithAwaitFn")
                    .field("protobuf_type", &protobuf_message.protobuf_type)
                    .field("protobuf_data", &protobuf_message.protobuf_data)
                    .finish()
            }
        }
    }
}


#[derive(Debug)]
/// Define the core message that will be used in the system
pub struct Message {
    pub message_type: MessageType,
}

#[derive(Debug)]
/// A request message that waits for a specific response
pub struct RequestWithAwaitMessage {
    pub message: ProtobufMessage,
    pub response_receiver: oneshot::Receiver<ProtobufMessage>, // Wait for a response
}

/// A request message that also includes a callback function to be executed on response
pub struct RequestWithAwaitFn {
    pub message: ProtobufMessage,
    pub response_receiver: oneshot::Receiver<ProtobufMessage>, // Wait for a response
    pub callback: Box<dyn FnOnce(ProtobufMessage) + Send + 'static>, // A callback function
}

impl std::fmt::Debug for RequestWithAwaitFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestWithAwaitFn")
            .field("protobuf_type", &self.message.protobuf_type)
            .field("protobuf_data", &self.message.protobuf_data)
            .finish()
    }
}

impl Message {
    // Helper method to access the embedded ProtobufMessage from a Message
    pub fn get_protobuf_message(&self) -> &ProtobufMessage {
        match &self.message_type {
            MessageType::Response { protobuf_message }
            | MessageType::Request { protobuf_message }
            | MessageType::RequestWithAwait { protobuf_message, .. }
            | MessageType::RequestWithAwaitFn { protobuf_message, .. } => protobuf_message,
        }
    }

    // Constructors for each message type
    pub fn new_response(protobuf_type: u32, protobuf_data: Vec<u8>) -> Self {
        Message {
            message_type: MessageType::Response {
                protobuf_message: ProtobufMessage {
                    protobuf_type,
                    protobuf_data,
                },
            },
        }
    }

    pub fn new_request(protobuf_type: u32, protobuf_data: Vec<u8>) -> Self {
        Message {
            message_type: MessageType::Request {
                protobuf_message: ProtobufMessage {
                    protobuf_type,
                    protobuf_data,
                },
            },
        }
    }

    pub fn new_request_with_await(
        protobuf_type: u32,
        protobuf_data: Vec<u8>,
        response_protobuf_type: u32,
    ) -> Self {
        Message {
            message_type: MessageType::RequestWithAwait {
                protobuf_message: ProtobufMessage {
                    protobuf_type,
                    protobuf_data,
                },
                response_protobuf_type,
            },
        }
    }

    pub fn new_request_with_await_fn(
        protobuf_type: u32,
        protobuf_data: Vec<u8>,
        response_protobuf_type: u32,
        callback: Box<dyn FnOnce(ProtobufMessage) + Send + 'static>,
    ) -> Self {
        Message {
            message_type: MessageType::RequestWithAwaitFn {
                protobuf_message: ProtobufMessage {
                    protobuf_type,
                    protobuf_data,
                },
                response_protobuf_type,
                callback,
            },
        }
    }
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

