use esphomeapi_manager::entity::{BaseEntity as _, Switch as RustSwitch};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct Switch {
  inner: RustSwitch,
}

impl Switch {
  pub fn new(rust_switch: RustSwitch) -> Self {
    Switch { inner: rust_switch }
  }
}

#[napi]
impl Switch {
  #[napi(getter)]
  pub fn key(&self) -> u32 {
    self.inner.key()
  }

  #[napi(getter)]
  pub fn name(&self) -> String {
    self.inner.name().to_string()
  }

  #[napi(getter)]
  pub fn is_on(&self) -> Result<bool> {
    match self.inner.get_state() {
      Ok(state) => Ok(state.state),
      Err(e) => Err(Error::new(Status::GenericFailure, e.to_string())),
    }
  }

  #[napi]
  pub async fn turn_on(&self) -> Result<()> {
    self
      .inner
      .turn_on()
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi]
  pub async fn turn_off(&self) -> Result<()> {
    self
      .inner
      .turn_off()
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi]
  pub async fn toggle(&self) -> Result<()> {
    self
      .inner
      .toggle()
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi]
  pub async fn set_state(&self, state: bool) -> Result<()> {
    self
      .inner
      .set_state(state)
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }
}
