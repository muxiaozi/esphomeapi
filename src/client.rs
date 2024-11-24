use protobuf::Message;

use crate::{
  model::{
    parse_user_service, EntityInfo, UserService, LIST_ENTITIES_SERVICES_RESPONSE_TYPES,
    SUBCRIBE_STATES_RESPONSE_TYPES,
  },
  utils::Options as _,
};
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

  pub async fn list_entities_services(&self) -> Result<(Vec<EntityInfo>, Vec<UserService>)> {
    let message = proto::api::ListEntitiesRequest::new();

    let entity_service_map = LIST_ENTITIES_SERVICES_RESPONSE_TYPES.clone();
    let mut response_protobuf_types: Vec<u32> = entity_service_map.keys().cloned().collect();
    // Add user defined services to the list of expected responses
    response_protobuf_types.push(proto::api::ListEntitiesServicesResponse::get_option_id());

    let response = self
      .connection
      .send_message_await_until(
        Box::new(message),
        response_protobuf_types,
        proto::api::ListEntitiesDoneResponse::get_option_id(),
        Duration::from_secs(60),
      )
      .await?;
    println!("Received list entities services response");

    let mut entities = Vec::new();
    let mut services = Vec::new();
    for message in response {
      let parser = entity_service_map
        .get(&message.protobuf_type)
        .ok_or_else(|| format!("Unknown message type: {}", message.protobuf_type))?;

      if message.protobuf_type == proto::api::ListEntitiesServicesResponse::get_option_id() {
        let parsed_service = parse_user_service(&message.protobuf_data)?;
        services.push(parsed_service);
      } else {
        let parsed_message = parser(&message.protobuf_data)?;
        entities.push(parsed_message);
      }
    }

    Ok((entities, services))
  }

  pub async fn subscribe_states(&mut self) -> Result<()> {
    let message = proto::api::SubscribeStatesRequest::new();

    let mut state_msg_types = SUBCRIBE_STATES_RESPONSE_TYPES
      .keys()
      .cloned()
      .collect::<Vec<u32>>();

    state_msg_types.push(proto::api::CameraImageResponse::get_option_id());

    for msg_type in state_msg_types {
      self.connection.add_message_handler(
        msg_type,
        Box::new(|_, msg| {
          println!("Received message: {:?}", msg);
          Ok(())
        }),
        false,
      );
    }

    self.connection.send_message(Box::new(message)).await?;
    Ok(())
  }

  pub async fn switch_command(&self, key: u32, state: bool) -> Result<()> {
    let message = proto::api::SwitchCommandRequest {
      key,
      state,
      ..Default::default()
    };

    self.connection.send_message(Box::new(message)).await?;
    Ok(())
  }
}
