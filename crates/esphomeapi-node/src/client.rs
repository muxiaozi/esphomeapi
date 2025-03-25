use esphomeapi::Client as RustClient;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::model::*;

#[napi]
pub struct Client {
  pub address: String,
  pub port: u32,
  pub password: Option<String>,
  pub expected_name: Option<String>,
  pub psk: Option<String>,
  pub client_info: Option<String>,
  pub keep_alive_duration: Option<u32>,
  inner: RustClient,
}

#[napi]
impl Client {
  #[napi(constructor)]
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
      address: address.clone(),
      port,
      password: password.clone(),
      expected_name: expected_name.clone(),
      psk: psk.clone(),
      client_info: client_info.clone(),
      keep_alive_duration,
      inner: RustClient::new(
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

  #[napi]
  pub async unsafe fn connect(&mut self, login: bool) -> Result<()> {
    self
      .inner
      .connect(login)
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e))
  }

  #[napi]
  pub async fn device_info(&self) -> Result<DeviceInfo> {
    self
      .inner
      .device_info()
      .await
      .map(|info| info.into())
      .map_err(|e| Error::new(Status::GenericFailure, e))
  }

  // #[napi]
  // pub async fn list_entities_services(&self) -> Result<(Vec<JsEntityInfo>, Vec<JsServiceInfo>)> {
  //   self
  //     .client
  //     .list_entities_services()
  //     .await
  //     .map(|(entities, services)| (entities.into(), services.into()))
  //     .map_err(|e| Error::new(Status::GenericFailure, e))
  // }

  // #[napi]
  // pub async unsafe fn disconnect(&mut self) -> Result<()> {}
}
