mod codec;

use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
  time::{Duration, SystemTime},
};

use bytes::Bytes;
use codec::{EspHomeCodec, EspHomeMessage, Noise, Plain};
use futures::SinkExt as _;
use protobuf::Message as _;
use tokio::{
  io::{AsyncWriteExt as _, BufReader, BufWriter},
  net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
  },
  task::JoinHandle,
  time::timeout,
};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::{proto, Error, Result};

use self::codec::FrameCodec;

use crate::utils::Options as _;
pub use codec::{Callback, ProtobufMessage};

#[derive(Clone, Debug)]
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

#[derive(Clone)]
pub struct Connection {
  host: String,
  port: u32,
  password: Option<String>,
  codec: EspHomeCodec,
  state: ConnectionState,
  is_connected: bool,
  keep_alive_duration: Duration,
  expected_name: Option<String>,
  client_info: String,
  message_handlers: Arc<RwLock<HashMap<u32, Vec<(bool, Callback)>>>>,
  channel_tx: Option<tokio::sync::mpsc::Sender<EspHomeMessage>>,
}

impl Connection {
  pub fn new(
    host: String,
    port: u32,
    password: Option<String>,
    expected_name: Option<String>,
    psk: Option<String>,
    client_info: Option<String>,
    keep_alive_duration: Option<u32>,
  ) -> Self {
    let codec = match psk {
      Some(psk) => EspHomeCodec::Noise(Arc::new(RwLock::new(Noise::new(
        psk,
        expected_name.clone(),
      )))),
      None => EspHomeCodec::Plain(Arc::new(RwLock::new(Plain::new()))),
    };

    // Initialize message handlers
    let mut message_handlers: HashMap<u32, Vec<(bool, Callback)>> = HashMap::new();
    proto::api::file_descriptor()
      .messages()
      .for_each(|msg_descriptor| {
        if let Some(message_options) = msg_descriptor.proto().options.as_ref() {
          match proto::api_options::exts::id.get(message_options) {
            Some(message_type) => {
              message_handlers.insert(message_type, Vec::new());
            }
            None => {}
          }
        }
      });

    Connection {
      host,
      port,
      password,
      codec,
      state: ConnectionState::Initialized,
      is_connected: false,
      keep_alive_duration: Duration::from_secs(keep_alive_duration.unwrap_or(20) as u64),
      expected_name,
      client_info: client_info.unwrap_or("esphome-rs".to_string()),
      message_handlers: Arc::new(RwLock::new(message_handlers)),
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

    self.add_message_handler(
      proto::api::DisconnectRequest::get_option_id(),
      Box::new(Self::handle_disconnect_request),
      false,
    );
    self.add_message_handler(
      proto::api::PingRequest::get_option_id(),
      Box::new(Self::handle_ping_request),
      false,
    );
    self.add_message_handler(
      proto::api::GetTimeRequest::get_option_id(),
      Box::new(Self::handle_get_time_request),
      false,
    );

    // Reading messages from TCP stream and sending them to the mpsc channel
    tokio::spawn(async move {
      let tx = tx.clone();
      loop {
        let frame = reader.next().await;
        match frame {
          Some(frame) => match frame {
            Ok(frame) => {
              tx.send(frame).await.unwrap();
            }
            Err(e) => {
              println!("Error reading frame: {:?}", e);
            }
          },
          None => {
            println!("Connection closed");
            break;
          }
        }
      }
      println!("Reader task finished");
    });

    let tx = self.channel_tx.clone().unwrap();
    let message_handlers = self.message_handlers.clone();
    let connection = Arc::new(RwLock::new(self.clone()));
    // Create a new codec for the writer
    let mut writer = FramedWrite::new(BufWriter::new(writer), self.codec.clone());

    // Spawn a new task to handle messages from the mpsc channel
    // This has to be spawned before any writing to the mpsc channel
    tokio::spawn(async move {
      while let Some(message) = rx.recv().await {
        match message.message_type {
          codec::EspHomeMessageType::Response { protobuf_message } => {
            println!("Received Response: {:?}", protobuf_message);
            if let Some(handlers) = message_handlers
              .write()
              .unwrap()
              .get_mut(&protobuf_message.protobuf_type)
            {
              handlers.retain(|(remove_after_call, callback)| {
                // Call the handler function with the message
                let _ = callback(
                  connection.clone(),
                  ProtobufMessage {
                    protobuf_type: protobuf_message.protobuf_type,
                    protobuf_data: protobuf_message.protobuf_data.clone(),
                  },
                );
                !*remove_after_call
              });
            }
          }
          codec::EspHomeMessageType::Request { protobuf_message } => {
            println!("Sending Request message {}", protobuf_message.protobuf_type);
            writer
              .send(EspHomeMessage::new_request(
                protobuf_message.protobuf_type,
                protobuf_message.protobuf_data,
              ))
              .await
              .unwrap();
          }
          codec::EspHomeMessageType::RequestWithAwait {
            protobuf_message,
            response_protobuf_type,
            tx: oneshot_tx,
          } => {
            println!(
              "Sending RequestWithAwait message {}",
              protobuf_message.protobuf_type
            );
            writer
              .send(EspHomeMessage::new_request(
                protobuf_message.protobuf_type,
                protobuf_message.protobuf_data,
              ))
              .await
              .unwrap();
            while let Some(message) = rx.recv().await {
              match message.message_type {
                codec::EspHomeMessageType::Response { protobuf_message } => {
                  if protobuf_message.protobuf_type == response_protobuf_type {
                    println!(
                      "Received Response for RequestWithAwait: {:?}",
                      protobuf_message
                    );

                    if let Err(_) = oneshot_tx.send(protobuf_message.clone()) {
                      println!("Error sending response to oneshot channel");
                    }

                    if let Some(handlers) = message_handlers
                      .write()
                      .unwrap()
                      .get_mut(&protobuf_message.protobuf_type)
                    {
                      handlers.retain(|(remove_after_call, callback)| {
                        // Call the handler function with the message

                        let _ = callback(
                          connection.clone(),
                          ProtobufMessage {
                            protobuf_type: protobuf_message.protobuf_type,
                            protobuf_data: protobuf_message.protobuf_data.clone(),
                          },
                        );
                        !*remove_after_call
                      });
                    }
                    break;
                  } else {
                    println!(
                      "Received ummatched Response for RequestWithAwait: {:?}",
                      protobuf_message
                    );
                    tx.send(EspHomeMessage::new_response(
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
          codec::EspHomeMessageType::RequestWithAwaitMultipleUntil {
            protobuf_message,
            response_protobuf_types,
            until_protobuf_type,
            tx: mpsc_tx,
          } => {
            println!(
              "Sending RequestWithAwaitMultipleUntil message {}",
              protobuf_message.protobuf_type
            );
            writer
              .send(EspHomeMessage::new_request(
                protobuf_message.protobuf_type,
                protobuf_message.protobuf_data,
              ))
              .await
              .unwrap();
            while let Some(message) = rx.recv().await {
              println!("Received message: {:?}", message);
              match message.message_type {
                codec::EspHomeMessageType::Response { protobuf_message } => {
                  if response_protobuf_types.contains(&protobuf_message.protobuf_type) {
                    println!(
                      "Received Response for RequestWithAwaitMultipleUntil: {:?}",
                      protobuf_message
                    );

                    if let Err(_) = mpsc_tx.send(protobuf_message.clone()).await {
                      println!("Error sending response to mpsc channel");
                    }

                    if let Some(handlers) = message_handlers
                      .write()
                      .unwrap()
                      .get_mut(&protobuf_message.protobuf_type)
                    {
                      handlers.retain(|(remove_after_call, callback)| {
                        // Call the handler function with the message
                        let _ = callback(
                          connection.clone(),
                          ProtobufMessage {
                            protobuf_type: protobuf_message.protobuf_type,
                            protobuf_data: protobuf_message.protobuf_data.clone(),
                          },
                        );
                        !*remove_after_call
                      });
                    }
                  } else if protobuf_message.protobuf_type == until_protobuf_type {
                    println!(
                      "Received until Response for RequestWithAwaitMultipleUntil: {:?}",
                      protobuf_message
                    );
                    break;
                  } else {
                    println!(
                      "Received ummatched Response for RequestWithAwaitMultipleUntil: {:?}",
                      protobuf_message
                    );
                    tx.send(EspHomeMessage::new_response(
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
        }
      }
    });

    self.init_hello(login).await?;
    self.state = ConnectionState::Connected;
    self.keep_alive(self.keep_alive_duration);

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
    let hello = self.make_hello_request();
    let expected_name = self.expected_name.clone();

    let response = self
      .send_message_await_response(Box::new(hello), proto::api::HelloResponse::get_option_id())
      .await?;
    let response = proto::api::HelloResponse::parse_from_bytes(&response.protobuf_data).unwrap();
    let received_name = response.name;
    if let Some(expected_name) = expected_name.clone() {
      if received_name != expected_name {
        println!("Received name does not match expected name");
      } else {
        println!("Received name matches expected name");
      }
    }

    if login {
      let connect = self.make_connect_request();
      self
        .send_message_await_response(
          Box::new(connect),
          proto::api::ConnectResponse::get_option_id(),
        )
        .await?;
    }
    Ok(())
  }

  pub async fn send_message(&self, message: Box<dyn protobuf::MessageDyn>) -> Result<()> {
    self.send_messages(vec![message]).await
  }

  pub async fn send_message_await_response(
    &self,
    message: Box<dyn protobuf::MessageDyn>,
    response_protobuf_type: u32,
  ) -> Result<ProtobufMessage> {
    let responses = self
      .send_messages_await_response(vec![message], vec![response_protobuf_type])
      .await?;
    if responses.len() != 1 {
      return Err("Expected exactly one response".into());
    }
    Ok(responses[0].clone())
  }

  pub async fn send_messages(&self, messages: Vec<Box<dyn protobuf::MessageDyn>>) -> Result<()> {
    let channel_tx = self.channel_tx.clone().unwrap();

    for message in messages {
      let protobuf_type = message
        .descriptor_dyn()
        .proto()
        .options
        .as_ref()
        .and_then(|options| proto::api_options::exts::id.get(options))
        .unwrap();
      let protobuf_data = message.write_to_bytes_dyn().unwrap();
      let request_message = EspHomeMessage::new_request(protobuf_type, protobuf_data);

      match channel_tx.send(request_message).await {
        Ok(_) => {}
        Err(e) => {
          return Err(format!("Error sending message: {:?}", e.0).into());
        }
      }
    }
    Ok(())
  }

  pub async fn send_messages_await_response(
    &self,
    messages: Vec<Box<dyn protobuf::MessageDyn>>,
    response_protobuf_types: Vec<u32>,
  ) -> Result<Vec<ProtobufMessage>> {
    if response_protobuf_types.len() != messages.len() {
      return Err("Number of response types must match number of messages".into());
    }

    let channel_tx = self.channel_tx.clone().unwrap();

    let mut responses = Vec::new();

    for (message, response_protobuf_type) in messages
      .into_iter()
      .zip(response_protobuf_types.into_iter())
    {
      let protobuf_type = message
        .descriptor_dyn()
        .proto()
        .options
        .as_ref()
        .and_then(|options| proto::api_options::exts::id.get(options))
        .unwrap();
      let protobuf_data = message.write_to_bytes_dyn().unwrap();
      let (tx, rx) = tokio::sync::oneshot::channel();
      let request_message = EspHomeMessage::new_request_with_await(
        protobuf_type,
        protobuf_data,
        response_protobuf_type,
        tx,
      );

      channel_tx.send(request_message).await?;

      match timeout(Duration::from_secs(5), rx).await {
        Ok(Ok(message)) => {
          responses.push(message);
        }
        Ok(Err(_)) => {
          return Err("Response channel closed".into());
        }
        Err(_) => {
          return Err("Timeout waiting for response".into());
        }
      }
    }
    Ok(responses)
  }

  pub async fn send_message_await_until(
    &self,
    message: Box<dyn protobuf::MessageDyn>,
    response_protobuf_types: Vec<u32>,
    until_protobuf_type: u32,
    timeout_duration: Duration,
  ) -> Result<Vec<ProtobufMessage>> {
    let channel_tx = self.channel_tx.clone().unwrap();

    let protobuf_type = message
      .descriptor_dyn()
      .proto()
      .options
      .as_ref()
      .and_then(|options| proto::api_options::exts::id.get(options))
      .unwrap();
    let protobuf_data = message.write_to_bytes_dyn().unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let request_message = EspHomeMessage::new_request_with_await_multiple_until(
      protobuf_type,
      protobuf_data,
      response_protobuf_types,
      until_protobuf_type,
      tx,
    );

    channel_tx.send(request_message).await?;

    let mut responses = Vec::new();

    while let Ok(Some(message)) = timeout(timeout_duration, rx.recv()).await {
      responses.push(message);
    }

    Ok(responses)
  }

  pub fn add_message_handler(&mut self, msg_type: u32, handler: Callback, remove_after_call: bool) {
    self
      .message_handlers
      .write()
      .unwrap()
      .entry(msg_type)
      .or_insert_with(Vec::new)
      .push((remove_after_call, handler));
  }

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

  fn keep_alive(&mut self, duration: Duration) -> JoinHandle<()> {
    let tx = self.channel_tx.clone().unwrap();
    let (protobuf_type, request) = proto::api::PingRequest::create_message_with_type();
    let protobuf_data = request.write_to_bytes().unwrap();

    let ping_response_protobuf_type = proto::api::PingResponse::get_option_id();
    self.add_message_handler(
      ping_response_protobuf_type,
      Box::new(move |_, message| {
        let response = proto::api::PingResponse::parse_from_bytes(&message.protobuf_data).unwrap();
        println!("Received PingResponse: {:?}", response);
        Ok(())
      }),
      false,
    );

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(duration);
      loop {
        interval.tick().await;
        println!("Sending PingRequest");
        if let Err(err) = tx
          .send(EspHomeMessage::new_request(
            protobuf_type,
            protobuf_data.clone(),
          ))
          .await
        {
          println!("Error sending PingRequest: {:?}", err);
        }
      }
    })
  }

  fn handle_disconnect_request(connection: Arc<RwLock<Self>>, _: ProtobufMessage) -> Result<()> {
    let mut connection = connection.write().unwrap();
    connection.state = ConnectionState::Closed;
    let message = proto::api::DisconnectResponse::default();
    let connection = connection.clone();
    tokio::spawn(async move {
      if let Err(e) = connection.send_message(Box::new(message)).await {
        println!("Error sending message: {:?}", e);
      }
    });
    Ok(())
  }

  fn handle_ping_request(connection: Arc<RwLock<Self>>, _: ProtobufMessage) -> Result<()> {
    let connection = connection.read().unwrap();
    let message = proto::api::PingResponse::default();
    let connection = connection.clone();
    tokio::spawn(async move {
      if let Err(e) = connection.send_message(Box::new(message)).await {
        println!("Error sending message: {:?}", e);
      }
    });
    Ok(())
  }

  fn handle_get_time_request(connection: Arc<RwLock<Self>>, _: ProtobufMessage) -> Result<()> {
    let connection = connection.read().unwrap();
    let mut response = proto::api::GetTimeResponse::new();
    response.epoch_seconds = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs() as u32;
    let connection = connection.clone();
    tokio::spawn(async move {
      if let Err(e) = connection.send_message(Box::new(response)).await {
        println!("Error sending message: {:?}", e);
      }
    });
    Ok(())
  }
}
