use std::{future, sync::Arc};

use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;

use crate::Error;

use self::codec::{EspHomeCodec, FrameCodec};

mod codec;

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
  codec: EspHomeCodec,
  state: ConnectionState,
  is_connected: bool,
  received_name: Option<String>,
}

impl Connection {
  pub fn new(host: String, port: u32, expected_name: Option<String>, psk: Option<String>) -> Self {

    let codec: EspHomeCodec = match psk {
      Some(psk) => EspHomeCodec::Noise(codec::Noise::new(psk, expected_name)),
      None => EspHomeCodec::Plain(codec::Plain::new())
    };

    Connection {
      host,
      port,
      codec,
      state: ConnectionState::Initialized,
      is_connected: false,
      received_name: None
    }
  }

  pub async fn connect(&mut self) -> Result<(), Error> {
    let mut stream = TcpStream::connect(format!("{}:{}", self.host, self.port)).await?;
    let (reader, mut writer) = stream.split();
    self.state = ConnectionState::SocketOpened;
    println!("Connected to {}:{}", self.host, self.port);

    let handshake_frame = self.codec.get_handshake_frame();
    let codec = std::mem::replace(&mut self.codec, EspHomeCodec::Plain(codec::Plain::new()));
    
    let mut reader = FramedRead::new(reader, codec);
    let mut writer = Arc::new(Mutex::new(writer));

    if let Some(frame) = handshake_frame {
      println!("Sending handshake frame");
      writer.lock().await.write_all(&frame).await?;
    }
    println!("Sent handshake frame");

    while let Some(message) = reader.next().await {
      let message = message?;
      println!("Received message: {:?}", message);
    }
  
    Ok(())
  }

  pub fn send_message<T>(&self, message: T) -> Result<(), ()> where T: prost::Message {
    self.send_messages(vec![message])
  }

  pub fn send_messages<T>(&self, messages: Vec<T>) -> Result<(), ()> where T: prost::Message {
    // self.frame_helper.send_messages(messages)
    Ok(())
  }
}