mod utils;
mod codec;

use std::{cell::RefCell, collections::HashMap, sync::{Arc, Mutex, RwLock}, vec};

use bytes::Bytes;
use codec::{EspHomeCodec, Noise, Plain };
use tokio::{io::{AsyncWriteExt, BufReader, BufWriter}, net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpStream}};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::sink::SinkExt;

use crate::{proto::{self, api::HelloResponse}, Error};

use self::codec::{ FrameCodec};
use utils::Options;

use crate::proto::api_options::exts::id;

enum ConnectionState {
  /// The connection is initialized, but connect() wasn't called yet
  Initialized, 
  /// The socket has been opened, but the handshake and login haven't been completed
  SocketOpened, 
  /// The handshake has been completed, messages can be exchanged
  HandshakeCompleted, 
  /// The connection has been established, authenticated data can be exchanged
  Connected, 
  Closed
}

pub struct Connection {
  host: String,
  port: u32,
  password: Option<String>,
  codec: EspHomeCodec,
  state: ConnectionState,
  is_connected: bool,
  received_name: Option<String>,
  client_info: String,
  message_handlers: HashMap<u32, Box<dyn FnMut(Vec<u8>) -> Result<(), ()>>>,
  channel_tx: Option<tokio::sync::mpsc::Sender<codec::Message>>,
}

impl Connection {
  pub fn new(host: String, port: u32, password: Option<String>, expected_name: Option<String>, psk: Option<String>, client_info: Option<String>) -> Self {

    let codec = match psk {
      Some(psk) => EspHomeCodec::Noise(Arc::new(RwLock::new(Noise::new(psk, expected_name)))),
      None => EspHomeCodec::Plain(Arc::new(RwLock::new(Plain::new())))
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
      message_handlers: HashMap::new(),
      channel_tx: None,
    }
  }

  pub async fn connect(&mut self, login: bool) -> Result<(), Error> {
    let stream = TcpStream::connect(format!("{}:{}", self.host, self.port)).await?;
    let (reader, mut writer) = stream.into_split();
    self.state = ConnectionState::SocketOpened;
    println!("Connected to {}:{}", self.host, self.port);

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    self.channel_tx = Some(tx.clone());
    
    let handshake_frame = self.codec.get_handshake_frame();
    
    let mut reader = FramedRead::new(BufReader::new(reader), self.codec.clone());

    self.init_handshake(handshake_frame, &mut reader, &mut writer).await?;
    self.init_hello(login).await?;

    self.state = ConnectionState::Connected;
    // Create a new codec for the writer
    let mut writer = FramedWrite::new(BufWriter::new(writer), self.codec.clone());
    

    let reader_handle = tokio::spawn(async move {
      let tx = tx.clone();
      while let Some(frame) = reader.next().await {
        match frame {
          Ok(frame) => {
            tx.send(frame).await.unwrap();
          },
          Err(e) => {
            println!("Error reading frame: {:?}", e);
          }
        }
      }
    });
    
    let mpsc_handle = tokio::spawn(async move {
      while let Some(message) = rx.recv().await {
        match message.message_type {
            codec::MessageType::Request => {
              let response_type = message.response_type.clone();
              writer.send(message).await.unwrap();
              if response_type.is_some() {
                let response_type = response_type.unwrap();
                loop {
                  let incoming_message = rx.recv().await.unwrap();
                  if response_type == incoming_message.protobuf_type {
                    break;
                  }
                }
              }
            }
            codec::MessageType::Response => {
              println!("Received response: {:?}", message);
              // let handler = self.message_handlers.get_mut(&message.protobuf_type);
            },
        }
      }
    });

    reader_handle.await.unwrap();
    mpsc_handle.await.unwrap();


    Ok(())
  }

  async fn init_handshake(&mut self, handshake_frame: Option<Bytes>, reader: &mut FramedRead<BufReader<OwnedReadHalf>, EspHomeCodec>, writer: &mut OwnedWriteHalf) -> Result<(), Error> {
    if let Some(handshake_frame) = handshake_frame {
      // Communication is encrypted
      writer.write_all(&handshake_frame).await?;
      let handshake_response = reader.next().await;
      if handshake_response.is_none() {
        return Err(Error::from(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "Handshake failed")));
      }
      let handshake_response = handshake_response.unwrap()?;
      if handshake_response.protobuf_type == 0 && handshake_response.protobuf_data == "Handshake completed".as_bytes().to_vec() {
        self.state = ConnectionState::HandshakeCompleted;
        println!("Handshake completed");
      }
    }
    self.is_connected = true;
    
    Ok(())
  }

  async fn init_hello(&mut self, login: bool) -> Result<(), Error> {
    let mut messages = Vec::<Box<dyn protobuf::MessageDyn>>::new();
    messages.push(Box::new(self.make_hello_request()));

    let mut response_types = vec![HelloResponse::get_option_id().unwrap()];
    if login {
      messages.push(Box::new(self.make_connect_request()));
      response_types.push(proto::api::ConnectResponse::get_option_id().unwrap());
    }

    self.send_messages(messages, Some(response_types)).await.unwrap();
    Ok(())
  }

  pub async fn send_message(&self) -> Result<(), ()>  {
    // self.frame_helper.send_message(message)
    println!("CSUMI");
    Ok(())
  }

  pub async fn send_messages(&self, messages: Vec<Box<dyn protobuf::MessageDyn>>, response_types: Option<Vec<u32>>) -> Result<(), ()> {
    
    let tx = self.channel_tx.as_ref().expect("Channel not initialized").clone();
    
    if response_types.is_some() && messages.len() != response_types.clone().unwrap().len() {
      return Err(());
    }

    for (idx, message) in messages.iter().enumerate() {
      let mut response_type: Option<u32> = None;
      if let Some(val) = response_types.clone() {
        response_type = Some(val.get(idx).unwrap().clone());
      }
      let message = codec::Message {
        message_type: codec::MessageType::Request,
        protobuf_type: message.descriptor_dyn().proto().options.as_ref().and_then(|options| id.get(options)).unwrap(),
        protobuf_data: message.write_to_bytes_dyn().unwrap(),
        response_type: response_type,
      };
      tx.send(message).await.unwrap();
    }
    Ok(())
  }

  // async fn send_messages_await_response(&self, messages: Vec<Box<dyn protobuf::MessageDyn>>, response_types: Vec<u32>) -> Result<(), ()> {
  //   self.send_messages(messages).await.unwrap();
  //   tokio::time::timeout(std::time::Duration::from_secs(5), async {
  //     let mut reader = self.reader.lock().await;
  //     loop {
  //       let message = reader.next().await.unwrap();
  //       if response_types.contains(&message.protobuf_type) {
  //         break;
  //       }
  //     }
  //   }).await.unwrap();
  //   Ok(())
  // }

  fn add_message_handler<T>(&mut self, msg_type: u32, handler: T) where T: FnMut(Vec<u8>) -> Result<(), ()> + 'static {
    // self.message_handlers.insert(msg_type, A::new(handler));
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

 
}