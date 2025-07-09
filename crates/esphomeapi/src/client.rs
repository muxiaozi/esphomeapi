use protobuf::{EnumOrUnknown, Message};

use crate::{
  connection::Callback,
  model::{
    parse_user_service, ColorMode, DeviceInfo, EntityInfo, UserService,
    LIST_ENTITIES_SERVICES_RESPONSE_TYPES,
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
    keep_alive_duration: Option<u32>,
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

  pub async fn device_info(&self) -> Result<DeviceInfo> {
    let message = proto::api::DeviceInfoRequest::default();

    let response = self
      .connection
      .send_message_await_response(
        Box::new(message),
        proto::api::DeviceInfoResponse::get_option_id(),
      )
      .await?;

    let response = proto::api::DeviceInfoResponse::parse_from_bytes(&response.protobuf_data)?;

    Ok(response.into())
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
      if message.protobuf_type == proto::api::ListEntitiesServicesResponse::get_option_id() {
        let parsed_service = parse_user_service(&message.protobuf_data)?;
        services.push(parsed_service);
      } else {
        let parser = entity_service_map
          .get(&message.protobuf_type)
          .ok_or_else(|| format!("Unknown message type: {}", message.protobuf_type))?;
        let parsed_message = parser(&message.protobuf_data)?;
        entities.push(parsed_message);
      }
    }

    Ok((entities, services))
  }

  pub fn add_message_handler(
    &mut self,
    msg_type: u32,
    callback: Callback,
    remove_after_call: bool,
  ) {
    self
      .connection
      .add_message_handler(msg_type, callback, remove_after_call);
  }

  pub async fn subscribe_states(&mut self) -> Result<()> {
    let message = proto::api::SubscribeStatesRequest::new();
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

  pub async fn light_command(
    &self,
    key: u32,
    state: Option<bool>,
    brightness: Option<f32>,
    color_mode: Option<ColorMode>,
    color_brightness: Option<f32>,
    rgb: Option<(f32, f32, f32)>,
    white: Option<f32>,
    color_temperature: Option<f32>,
    cold_white: Option<f32>,
    warm_white: Option<f32>,
    transition_length: Option<f32>,
    flash_length: Option<f32>,
    effect: Option<String>,
  ) -> Result<()> {
    let message = proto::api::LightCommandRequest {
      key,
      has_state: state.is_some(),
      state: state.unwrap_or_default(),
      has_brightness: brightness.is_some(),
      brightness: brightness.unwrap_or_default(),
      has_color_mode: color_mode.is_some(),
      color_mode: EnumOrUnknown::new(color_mode.unwrap_or_default().into()),
      has_color_brightness: color_brightness.is_some(),
      color_brightness: color_brightness.unwrap_or_default(),
      has_rgb: rgb.is_some(),
      red: rgb.unwrap_or_default().0,
      green: rgb.unwrap_or_default().1,
      blue: rgb.unwrap_or_default().2,
      has_white: white.is_some(),
      white: white.unwrap_or_default(),
      has_color_temperature: color_temperature.is_some(),
      color_temperature: color_temperature.unwrap_or_default(),
      has_cold_white: cold_white.is_some(),
      cold_white: cold_white.unwrap_or_default(),
      has_warm_white: warm_white.is_some(),
      warm_white: warm_white.unwrap_or_default(),
      has_transition_length: transition_length.is_some(),
      transition_length: (transition_length.unwrap_or_default() * 1000.0).round() as u32,
      has_flash_length: flash_length.is_some(),
      flash_length: (flash_length.unwrap_or_default() * 1000.0).round() as u32,
      has_effect: effect.is_some(),
      effect: effect.unwrap_or_default(),
      ..Default::default()
    };

    self.connection.send_message(Box::new(message)).await?;
    Ok(())
  }
}
