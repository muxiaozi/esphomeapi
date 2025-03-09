mod model;

use esphomeapi::Client;
use model::JsDeviceInfo;
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(js_name = "Client")]
pub struct JsClient {
  client: Client,
}

#[napi]
impl JsClient {
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
      client: Client::new(
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
      .client
      .connect(login)
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e))
  }

  #[napi]
  pub async fn device_info(&self) -> Result<JsDeviceInfo> {
    self
      .client
      .device_info()
      .await
      .map(|info| info.into())
      .map_err(|e| Error::new(Status::GenericFailure, e))
  }
  // #[napi]
  // pub async unsafe fn disconnect(&mut self) -> Result<()> {}
}
