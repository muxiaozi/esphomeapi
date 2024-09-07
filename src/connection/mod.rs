mod codec;
mod utils;

use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use bytes::Bytes;
use codec::{EspHomeCodec, Message, Noise, Plain, ProtobufMessage};
use futures::SinkExt as _;
use tokio::{
  io::{AsyncWriteExt, BufReader, BufWriter},
  net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
  },
};
use tokio_stream::{Stream, StreamExt};
use tokio_util::codec::{FramedRead, FramedWrite};
use utils::Options as _;

use crate::{proto, Error, Result};

use self::codec::FrameCodec;

enum ConnectionState {
  /// The connection is initialized, but connect() wasn't called yet
  Initialized,
  /// The socket has been opened, but the handshake and login haven't been completed
  SocketOpened,
  /// The handshake has been completed, messages can be exchanged
  HandshakeCompleted,
  /// The connection has been established, authenticated data can be exchanged
  Connected,
  Closed,
}

type ResponseHandler = Box<dyn Fn(Vec<u8>) -> Result<()> + Send + Sync + 'static>;

pub struct Connection {
  host: String,
  port: u32,
  password: Option<String>,
  codec: EspHomeCodec,
  state: ConnectionState,
  is_connected: bool,
  received_name: Option<String>,
  client_info: String,
  message_handlers: Arc<RwLock<HashMap<u32, ResponseHandler>>>,
  channel_tx: Option<tokio::sync::mpsc::Sender<codec::Message>>,
}

impl Connection {
  pub fn new(
    host: String,
    port: u32,
    password: Option<String>,
    expected_name: Option<String>,
    psk: Option<String>,
    client_info: Option<String>,
  ) -> Self {
    let codec = match psk {
      Some(psk) => EspHomeCodec::Noise(Arc::new(RwLock::new(Noise::new(psk, expected_name)))),
      None => EspHomeCodec::Plain(Arc::new(RwLock::new(Plain::new()))),
    };

    Connection {
      host,
      port,
      password,
      codec,
      state: ConnectionState::Initialized,
      is_connected: false,
      received_name: None,
      client_info: client_info.unwrap_or("esphome-rs".to_string()),
      message_handlers: Arc::new(RwLock::new(HashMap::new())),
      channel_tx: None,
    }
  }

  pub async fn connect(&mut self, login: bool) -> Result<()> {
    let stream = TcpStream::connect(format!("{}:{}", self.host, self.port)).await?;
    let (reader, mut writer) = stream.into_split();
    self.state = ConnectionState::SocketOpened;
    println!("Connected to {}:{}", self.host, self.port);

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    self.channel_tx = Some(tx.clone());

    let handshake_frame = self.codec.get_handshake_frame();

    let mut reader = FramedRead::new(BufReader::new(reader), self.codec.clone());

    self
      .init_handshake(handshake_frame, &mut reader, &mut writer)
      .await?;
    self.init_hello(login).await?;

    self.state = ConnectionState::Connected;
    // Create a new codec for the writer
    let mut writer = FramedWrite::new(BufWriter::new(writer), self.codec.clone());

    let tcp_reader_handle = tokio::spawn(async move {
      let tx = tx.clone();
      while let Some(frame) = reader.next().await {
        match frame {
          Ok(frame) => {
            tx.send(frame).await.unwrap();
          }
          Err(e) => {
            println!("Error reading frame: {:?}", e);
          }
        }
      }
    });

    let message_handlers = self.message_handlers.clone();
    let tx = self.channel_tx.clone().unwrap();

    let mpsc_handle = tokio::spawn(async move {
      while let Some(message) = rx.recv().await {
        match message.message_type {
          codec::MessageType::Response { protobuf_message } => {
            println!("Received Response: {:?}", protobuf_message);
            let mut handlers = message_handlers.write().unwrap();
            if let Some(handler) = handlers.get(&protobuf_message.protobuf_type) {
              if let Err(e) = handler(protobuf_message.protobuf_data) {
                println!("Error handling message: {:?}", e);
              } else {
                handlers.remove(&protobuf_message.protobuf_type);
              }
            }
          }
          codec::MessageType::Request { protobuf_message } => {
            println!("Sending Request message {}", protobuf_message.protobuf_type);
            writer
              .send(Message::new_request(
                protobuf_message.protobuf_type,
                protobuf_message.protobuf_data,
              ))
              .await
              .unwrap();
          }
          codec::MessageType::RequestWithAwait {
            protobuf_message,
            response_protobuf_type,
          } => {
            println!(
              "Sending RequestWithAwait message {}",
              protobuf_message.protobuf_type
            );
            writer
              .send(Message::new_request(
                protobuf_message.protobuf_type,
                protobuf_message.protobuf_data,
              ))
              .await
              .unwrap();
            while let Some(message) = rx.recv().await {
              match message.message_type {
                codec::MessageType::Response { protobuf_message } => {
                  if protobuf_message.protobuf_type == response_protobuf_type {
                    println!("Received Response for RequestWithAwait: {:?}", protobuf_message);
                    let mut handlers = message_handlers.write().unwrap();
                    if let Some(handler) = handlers.get(&protobuf_message.protobuf_type) {
                      if let Err(e) = handler(protobuf_message.protobuf_data) {
                        println!("Error handling message: {:?}", e);
                      } else {
                        handlers.remove(&protobuf_message.protobuf_type);
                      }
                    }
                    break;
                  } else {
                    println!("Received ummatched Response for RequestWithAwait: {:?}", protobuf_message);
                    tx.send(Message::new_response(
                      protobuf_message.protobuf_type,
                      protobuf_message.protobuf_data,
                    ))
                    .await
                    .unwrap();
                  }
                }
                _ => {}
              }
            }
          }
          codec::MessageType::RequestWithAwaitFn {
            protobuf_message,
            response_protobuf_type,
            callback,
          } => {
            println!(
              "Sending RequestWithAwaitFn message {}",
              protobuf_message.protobuf_type
            );
            let send_proto = protobuf_message.clone();
            writer
              .send(Message::new_request(
                send_proto.protobuf_type,
                send_proto.protobuf_data,
              ))
              .await
              .unwrap();
            while let Some(message) = rx.recv().await {
              match message.message_type {
                codec::MessageType::Response {
                  protobuf_message: response_message,
                } => {
                  if response_message.protobuf_type == response_protobuf_type {
                    callback(response_message);
                    break;
                  } else {
                    tx.send(Message::new_response(
                      response_message.protobuf_type,
                      response_message.protobuf_data,
                    ))
                    .await
                    .unwrap();
                  }
                }
                _ => {}
              }
            }
          }
        }
      }
    });

    tcp_reader_handle.await.unwrap();
    mpsc_handle.await.unwrap();

    Ok(())
  }

  async fn init_handshake(
    &mut self,
    handshake_frame: Option<Bytes>,
    reader: &mut FramedRead<BufReader<OwnedReadHalf>, EspHomeCodec>,
    writer: &mut OwnedWriteHalf,
  ) -> Result<()> {
    if let Some(handshake_frame) = handshake_frame {
      // Communication is encrypted
      writer.write_all(&handshake_frame).await?;
      let handshake_response = reader.next().await;
      if handshake_response.is_none() {
        return Err(Error::from(std::io::Error::new(
          std::io::ErrorKind::ConnectionAborted,
          "Handshake failed",
        )));
      }
      let handshake_message_response = handshake_response.unwrap()?;
      let handshake_protobuf = handshake_message_response.get_protobuf_message();
      if handshake_protobuf.protobuf_type == 0
        && handshake_protobuf.protobuf_data == "Handshake completed".as_bytes().to_vec()
      {
        self.state = ConnectionState::HandshakeCompleted;
        println!("Handshake completed");
      }
    }
    self.is_connected = true;

    Ok(())
  }

  async fn init_hello(&mut self, login: bool) -> Result<()> {
    let mut messages = Vec::<Box<dyn protobuf::MessageDyn>>::new();
    messages.push(Box::new(self.make_hello_request()));

    let mut response_protobuf_types = vec![proto::api::HelloResponse::get_option_id()];
    if login {
      messages.push(Box::new(self.make_connect_request()));
      response_protobuf_types.push(proto::api::ConnectResponse::get_option_id());
    }

    self
      .send_messages_await_response(messages, response_protobuf_types)
      .await
      .unwrap();

    Ok(())
  }

  pub async fn send_message(&self) -> Result<()> {
    // self.frame_helper.send_message(message)
    println!("CSUMI");
    Ok(())
  }

  pub async fn send_messages(&self, messages: Vec<Box<dyn protobuf::MessageDyn>>) -> Result<()> {
    let tx = self.channel_tx.clone().unwrap();

    for message in messages {
      let protobuf_type = message
        .descriptor_dyn()
        .proto()
        .options
        .as_ref()
        .and_then(|options| proto::api_options::exts::id.get(options))
        .unwrap();
      let protobuf_data = message.write_to_bytes_dyn().unwrap();
      let request_message = Message::new_request(protobuf_type, protobuf_data);

      match tx.send(request_message).await {
        Ok(_) => {}
        Err(e) => {
          return Err(
            Error::from(std::io::Error::new(
              std::io::ErrorKind::BrokenPipe,
              format!("Error sending message: {:?}", e.0),
            ))
            .into(),
          );
        }
      }
    }
    Ok(())
  }

  pub async fn send_messages_await_response(
    &self,
    messages: Vec<Box<dyn protobuf::MessageDyn>>,
    response_protobuf_types: Vec<u32>,
  ) -> Result<()> {
    if response_protobuf_types.len() != messages.len() {
      return Err(
        Error::from(std::io::Error::new(
          std::io::ErrorKind::InvalidInput,
          "Number of response types must match number of messages",
        ))
        .into(),
      );
    }

    let tx = self.channel_tx.clone().unwrap();
    for (idx, message) in messages.iter().enumerate() {
      let protobuf_type = message
        .descriptor_dyn()
        .proto()
        .options
        .as_ref()
        .and_then(|options| proto::api_options::exts::id.get(options))
        .unwrap();
      let protobuf_data = message.write_to_bytes_dyn().unwrap();
      let request_message =
        Message::new_request_with_await(protobuf_type, protobuf_data, response_protobuf_types[idx]);

      match tx.send(request_message).await {
        Ok(_) => {}
        Err(e) => {
          return Err(
            Error::from(std::io::Error::new(
              std::io::ErrorKind::BrokenPipe,
              format!("Error sending message: {:?}", e.0),
            ))
            .into(),
          );
        }
      }
    }
    Ok(())
  }

  pub async  fn send_messages_await_response_callback(
    &self,
    messages: Vec<Box<dyn protobuf::MessageDyn>>,
    response_protobuf_types: Vec<u32>,
    callback: Box<dyn FnOnce(ProtobufMessage) + Send + 'static>,
  ) -> Result<()> {
    if response_protobuf_types.len() != messages.len() {
      return Err(
        Error::from(std::io::Error::new(
          std::io::ErrorKind::InvalidInput,
          "Number of response types must match number of messages",
        ))
        .into(),
      );
    }

    let tx = self.channel_tx.clone().unwrap();
    for (idx, message) in messages.iter().enumerate() {
      let protobuf_type = message
        .descriptor_dyn()
        .proto()
        .options
        .as_ref()
        .and_then(|options| proto::api_options::exts::id.get(options))
        .unwrap();
      let protobuf_data = message.write_to_bytes_dyn().unwrap();
      let request_message =
        Message::new_request_with_await_fn(protobuf_type, protobuf_data, response_protobuf_types[idx], callback.clone());

      match tx.send(request_message).await {
        Ok(_) => {}
        Err(e) => {
          return Err(
            Error::from(std::io::Error::new(
              std::io::ErrorKind::BrokenPipe,
              format!("Error sending message: {:?}", e.0),
            ))
            .into(),
          );
        }
      }
    }
    Ok(())
  }

  // fn add_message_handler<T>(&mut self, msg_type: u32, handler: T) where T: FnMut(Vec<u8>) -> Result<(), ()> + 'static {
  //   self.message_handlers.insert(msg_type, Box::new(handler));
  // }

  fn make_hello_request(&self) -> proto::api::HelloRequest {
    let mut request = proto::api::HelloRequest::default();
    request.client_info = self.client_info.clone();
    request.api_version_major = 1;
    request.api_version_minor = 10;
    request
  }

  fn make_connect_request(&self) -> proto::api::ConnectRequest {
    let mut request = proto::api::ConnectRequest::default();
    if self.password.is_some() {
      request.password = self.password.clone().unwrap();
    };
    request
  }

  fn keep_alive(&self) -> proto::api::PingRequest {
    proto::api::PingRequest::default()
  }
}
