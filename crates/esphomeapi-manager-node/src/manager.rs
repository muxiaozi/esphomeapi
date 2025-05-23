use esphomeapi_manager::Manager as RustManager;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

use crate::entity::{self, Switch};

#[napi(object)]
pub struct ConnectionOptions {
  pub address: String,
  pub port: u32,
  pub password: Option<String>,
  pub expected_name: Option<String>,
  pub psk: Option<String>,
  pub client_info: Option<String>,
  pub keep_alive_duration: Option<u32>,
}

#[napi(object)]
pub struct EntityInfo {
  pub key: u32,
  pub name: String,
  pub unique_id: String,
  pub object_id: String,
  pub device_class: String,
  pub disabled_by_default: bool,
  pub entity_category: String,
  pub icon: String,
}

#[napi]
pub struct Manager {
  inner: RustManager,
}

#[napi]
impl Manager {
  #[napi(factory)]
  pub async fn connect(options: ConnectionOptions) -> Result<Manager> {
    let manager = RustManager::new(
      options.address,
      options.port,
      options.password,
      options.expected_name,
      options.psk,
      options.client_info,
      options.keep_alive_duration,
    )
    .await;

    Ok(Manager { inner: manager })
  }

  #[napi]
  pub fn get_device_name(&self) -> String {
    self.inner.device_info.name.clone()
  }

  #[napi]
  pub fn get_device_mac(&self) -> String {
    self.inner.device_info.mac_address.clone()
  }

  #[napi]
  pub fn get_switches(&self) -> Vec<entity::Switch> {
    self
      .inner
      .get_switches()
      .iter()
      .map(|s| Switch::new(s.clone().into()))
      .collect()
  }
}
