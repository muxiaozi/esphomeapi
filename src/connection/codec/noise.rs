use std::io::Error;

use base64::prelude::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use noise_protocol::{patterns::noise_nn_psk0, CipherState, HandshakeState};
use noise_rust_crypto::{ChaCha20Poly1305, Sha256, X25519};
use tokio_util::codec::{Decoder, Encoder};

use super::{FrameCodec, Message};

static PROLOGUE: &'static [u8] = b"NoiseAPIInit\x00\x00";
static HELLO: &'static [u8] = &[0x01, 0x00, 0x00];

#[derive(PartialEq, Debug)]
enum NoiseState {
  Hello,
  Handshake,
  Ready,
  Closed,
}

pub struct Noise {
  state: NoiseState,
  expected_server_name: Option<String>,
  initiator: Option<HandshakeState<X25519, ChaCha20Poly1305, Sha256>>,
  decoder: Option<CipherState<ChaCha20Poly1305>>,
  encoder: Option<CipherState<ChaCha20Poly1305>>,
  buffer: BytesMut,
}

impl Noise {
  pub fn new(psk: String, expected_server_name: Option<String>) -> Self {

    let base64_psk = BASE64_STANDARD.decode(psk.as_bytes()).unwrap();
    let mut initiator = HandshakeState::new(noise_nn_psk0(), true, PROLOGUE, None, None, None, None);
    initiator.push_psk(base64_psk.as_slice());

    Noise {
      state: NoiseState::Hello,
      expected_server_name,
      initiator: Some(initiator),
      decoder: None,
      encoder: None,
      buffer: BytesMut::with_capacity(1024),
    }
  }
}

impl FrameCodec for Noise {
  fn get_handshake_frame(&mut self) -> Option<Bytes> {
    let buffer = self.initiator.as_mut().unwrap().write_message_vec(&[]).unwrap();
    let len = buffer.len() + 1;
    let header = [0x01, (len.checked_shr(8).unwrap_or(0)) as u8, len as u8].to_vec();

    let mut frame = BytesMut::with_capacity(1024);
    frame.extend_from_slice(HELLO);
    frame.extend_from_slice(&header);
    frame.put_u8(0);
    frame.extend_from_slice(&buffer);

    Some(frame.freeze())
  }

  fn parse_frame(&self, src: &mut bytes::BytesMut) -> Result<(u8, u8), Error> {
    let header = &src[..3];

    let preamble = header[0];
    if preamble != 0x01 {
      return Err(Error::new(std::io::ErrorKind::InvalidData, "Invalid preamble"));
    }

    let msg_size_high = header[1];
    let msg_size_low = header[2];

    src.advance(3);
    if src.len() < (msg_size_high as usize).checked_shl(8).unwrap_or(0) | msg_size_low as usize {
      return Err(Error::new(std::io::ErrorKind::InvalidData, "Invalid message size"));
    }

    Ok((msg_size_high, msg_size_low))
  }
}

impl Decoder for Noise {
  type Item = Message;
  type Error = std::io::Error;

  fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if src.is_empty() {
      return Ok(None);
    }

    let (msg_len_high, msg_len_low) = self.parse_frame(src)?;
    let msg_len = (msg_len_high as usize).checked_shl(8).unwrap_or(0) | msg_len_low as usize;

    let mut msg = src.split_to(msg_len);

    match self.state {
      NoiseState::Hello => {
        let chosen_proto = msg[0];
        if chosen_proto != 0x01 {
          return Err(Error::new(std::io::ErrorKind::InvalidData, "Invalid protocol"));
        }

        let server_name_i = msg.iter().skip(1).position(|&x| x == 0x00);

        match server_name_i {
          Some(server_name_i) => {
            // server name found, this extension was added in 2022.2
            let server_name = msg.iter().skip(1).take(server_name_i).copied().collect::<Vec<u8>>();
            let server_name = String::from_utf8(server_name).unwrap();

            if let Some(expected_server_name) = &self.expected_server_name {
              if server_name != *expected_server_name {
                return Err(Error::new(std::io::ErrorKind::InvalidData, "Invalid server name"));
              }
            }
          },
          None => (), // server name not found
        }
        self.state = NoiseState::Handshake;
      },
      NoiseState::Handshake => {
        if msg[0] != 0x00 {
          return Err(Error::new(std::io::ErrorKind::InvalidData, "Invalid preamble"));
        }
        msg.advance(1);

        let mut handshake_state = self.initiator.take().unwrap();
        handshake_state.read_message_vec(&msg).unwrap();

        if handshake_state.completed() {
          let (encoder, decoder) = handshake_state.get_ciphers();
          self.encoder = Some(encoder);
          self.decoder = Some(decoder);
          self.state = NoiseState::Ready;
          println!("Noise handshake completed")
        } else {
          self.initiator = Some(handshake_state);
        }
      },
      NoiseState::Ready => {
        if self.decoder.is_none() {
          return Err(Error::new(std::io::ErrorKind::InvalidData, "Decoder not initialized"));
        }
        let buffer = self.decoder.as_mut().unwrap().decrypt_vec(&msg).unwrap();

        // Message layout is
        // 2 bytes: message type
        // 2 bytes: message length
        // N bytes: message data
        let msg_type_high = buffer[0];
        let msg_type_low = buffer[1];

        return Ok(Some(Message {
          protobuf_type: (msg_type_high).checked_shr(8).unwrap_or(0) | msg_type_low,
          protobuf_data: buffer[4..].to_vec(),
        }));
      },
      NoiseState::Closed => return Err(Error::new(std::io::ErrorKind::InvalidData, "Connection closed")),
      _ => {}
    }

    Ok(None)
  }
}

impl Encoder<Message> for Noise {
  type Error = std::io::Error;

  fn encode(&mut self, item: Message, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {

    if self.state != NoiseState::Ready || self.encoder.is_none() {
      return Err(Error::new(std::io::ErrorKind::InvalidData, "Encoder not initialized"));
    }

    let mut frame = BytesMut::with_capacity(1024);
    let mut buffer = BytesMut::new();

    let data_len = item.protobuf_data.len() as u8;
    let data_header = [(item.protobuf_type.checked_shr(8).unwrap_or(0)) as u8, item.protobuf_type as u8, (data_len.checked_shr(8).unwrap_or(0)) as u8, data_len as u8].to_vec();
    buffer.extend_from_slice(&data_header);
    buffer.extend_from_slice(&item.protobuf_data);

    self.encoder.as_mut().unwrap().encrypt(&buffer, &mut frame);
    let len = frame.len();
    let header = [0x01, (len.checked_shr(8).unwrap_or(0)) as u8, len as u8].to_vec();

    dst.extend_from_slice(&header);
    dst.extend_from_slice(&frame);

    Ok(())
  }
}
