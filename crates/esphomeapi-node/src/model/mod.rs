use napi_derive::napi;

#[napi(js_name = "DeviceInfo")]
pub struct JsDeviceInfo(esphomeapi::model::DeviceInfo);

#[napi]
impl JsDeviceInfo {
  #[napi(constructor)]
  pub fn new() -> Self {
    JsDeviceInfo(esphomeapi::model::DeviceInfo::default())
  }

  #[napi(getter)]
  pub fn uses_password(&self) -> bool {
    self.0.uses_password
  }

  #[napi(getter)]
  pub fn name(&self) -> String {
    self.0.name.clone()
  }

  #[napi(getter)]
  pub fn friendly_name(&self) -> String {
    self.0.friendly_name.clone()
  }

  #[napi(getter)]
  pub fn mac_address(&self) -> String {
    self.0.mac_address.clone()
  }

  #[napi(getter)]
  pub fn compilation_time(&self) -> String {
    self.0.compilation_time.clone()
  }

  #[napi(getter)]
  pub fn model(&self) -> String {
    self.0.model.clone()
  }

  #[napi(getter)]
  pub fn manufacturer(&self) -> String {
    self.0.manufacturer.clone()
  }

  #[napi(getter)]
  pub fn has_deep_sleep(&self) -> bool {
    self.0.has_deep_sleep
  }

  #[napi(getter)]
  pub fn esphome_version(&self) -> String {
    self.0.esphome_version.clone()
  }

  #[napi(getter)]
  pub fn project_name(&self) -> String {
    self.0.project_name.clone()
  }

  #[napi(getter)]
  pub fn project_version(&self) -> String {
    self.0.project_version.clone()
  }

  #[napi(getter)]
  pub fn webserver_port(&self) -> u32 {
    self.0.webserver_port
  }

  #[napi(getter)]
  pub fn legacy_voice_assistant_version(&self) -> u32 {
    self.0.legacy_voice_assistant_version
  }

  #[napi(getter)]
  pub fn voice_assistant_feature_flags(&self) -> u32 {
    self.0.voice_assistant_feature_flags
  }

  #[napi(getter)]
  pub fn legacy_bluetooth_proxy_version(&self) -> u32 {
    self.0.legacy_bluetooth_proxy_version
  }

  #[napi(getter)]
  pub fn bluetooth_proxy_feature_flags(&self) -> u32 {
    self.0.bluetooth_proxy_feature_flags
  }

  #[napi(getter)]
  pub fn suggested_area(&self) -> String {
    self.0.suggested_area.clone()
  }
}

impl From<esphomeapi::model::DeviceInfo> for JsDeviceInfo {
  fn from(info: esphomeapi::model::DeviceInfo) -> Self {
    JsDeviceInfo(info)
  }
}
