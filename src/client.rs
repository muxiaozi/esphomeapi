use protobuf::Message;

use crate::{connection::ProtobufMessage, utils::Options as _};
use std::time::Duration;

use crate::{connection::Connection, proto, Result};

pub struct Client {
  connection: Connection,
}

impl Client {
  pub fn new(
    address: String,
    port: u32,
    password: Option<String>,
    expected_name: Option<String>,
    psk: Option<String>,
    client_info: Option<String>,
    keep_alive_duration: Option<Duration>,
  ) -> Self {
    Self {
      connection: Connection::new(
        address,
        port,
        password,
        expected_name,
        psk,
        client_info,
        keep_alive_duration,
      ),
    }
  }

  pub async fn connect(&mut self, login: bool) -> Result<()> {
    self.connection.connect(login).await
  }

  pub async fn device_info(&mut self) -> Result<()> {
    let message = proto::api::DeviceInfoRequest::default();

    println!("Sending device info request");
    let response = self
      .connection
      .send_message_await_response(
        Box::new(message),
        proto::api::DeviceInfoResponse::get_option_id(),
      )
      .await?;
    println!("Received device info response");

    let response = proto::api::DeviceInfoResponse::parse_from_bytes(&response.protobuf_data)?;

    println!("Device info: {:?}", response);
    Ok(())
  }
}
