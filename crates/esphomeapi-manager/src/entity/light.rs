use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use esphomeapi::{
  Client,
  model::{EntityState, LightInfo, LightState},
};

use super::{BaseEntity, StateError, StateResult};

#[derive(Clone)]
pub struct Light {
  client: Arc<Client>,
  info: LightInfo,
  states: Arc<RwLock<HashMap<u32, EntityState>>>,
}

impl Light {
  pub fn new(
    client: Arc<Client>,
    info: LightInfo,
    states: Arc<RwLock<HashMap<u32, EntityState>>>,
  ) -> Self {
    Light {
      client,
      info,
      states,
    }
  }

  pub fn get_state(&self) -> StateResult<LightState> {
    let states_guard = self.states.read().unwrap();
    let state = states_guard
      .get(&self.info.entity_info.key)
      .ok_or(StateError::EntityKeyNotFound(self.info.entity_info.key));

    match state? {
      EntityState::Light(state) => Ok(state.clone()),
      _ => Err(StateError::NotValidState),
    }
  }

  pub fn is_on(&self) -> esphomeapi::Result<bool> {
    let state = self.get_state()?;

    Ok(state.state)
  }

  pub async fn turn_on(&self) -> esphomeapi::Result<()> {
    self
      .client
      .light_command(
        self.info.entity_info.key,
        Some(true),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
      )
      .await
  }

  pub async fn turn_off(&self) -> esphomeapi::Result<()> {
    self
      .client
      .light_command(
        self.info.entity_info.key,
        Some(false),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
      )
      .await
  }

  pub async fn toggle(&self) -> esphomeapi::Result<()> {
    match self.is_on()? {
      true => self.turn_off().await,
      false => self.turn_on().await,
    }
  }

  pub fn brightness(&self) -> esphomeapi::Result<f32> {
    let state = self.get_state()?;

    Ok(state.brightness)
  }
}

impl BaseEntity for Light {
  fn key(&self) -> u32 {
    self.info.entity_info.key
  }

  fn name(&self) -> String {
    self.info.entity_info.name.clone()
  }
}
