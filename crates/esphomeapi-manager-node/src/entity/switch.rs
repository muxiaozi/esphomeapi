use esphomeapi_manager::entity::{Entity as _, Switch as RustSwitch};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

// use super::super::{EntityInfo, SimpleState};

#[napi]
pub struct Switch {
  inner: Arc<RustSwitch>,
}

impl Switch {
  pub fn new(rust_switch: Arc<RustSwitch>) -> Self {
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

  #[napi]
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
    match self.is_on()? {
      true => self.turn_off().await,
      false => self.turn_on().await,
    }
  }
}
