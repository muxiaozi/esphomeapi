use esphomeapi_manager::entity::{BaseEntity as _, Light as RustLight};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct Light {
  inner: RustLight,
}

impl Light {
  pub fn new(rust_light: RustLight) -> Self {
    Light { inner: rust_light }
  }
}

#[napi]
impl Light {
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
    self
      .inner
      .is_on()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
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
}
