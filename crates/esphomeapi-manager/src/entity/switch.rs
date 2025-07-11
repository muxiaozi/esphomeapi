use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use esphomeapi::{
  Client,
  model::{EntityState, SwitchInfo, SwitchState},
};

use super::{BaseEntity, StateError, StateResult};

#[derive(Clone)]
pub struct Switch {
  client: Arc<Client>,
  info: SwitchInfo,
  states: Arc<RwLock<HashMap<u32, EntityState>>>,
}

impl Switch {
  pub fn new(
    client: Arc<Client>,
    info: SwitchInfo,
    states: Arc<RwLock<HashMap<u32, EntityState>>>,
  ) -> Self {
    Switch {
      client,
      info,
      states,
    }
  }

  pub fn get_state(&self) -> StateResult<SwitchState> {
    let states_guard = self.states.read().unwrap();
    let state = states_guard
      .get(&self.info.entity_info.key)
      .ok_or(StateError::EntityKeyNotFound(self.info.entity_info.key));

    match state? {
      EntityState::Switch(state) => Ok(state.clone()),
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
      .switch_command(self.info.entity_info.key, true)
      .await
  }

  pub async fn turn_off(&self) -> esphomeapi::Result<()> {
    self
      .client
      .switch_command(self.info.entity_info.key, false)
      .await
  }

  pub async fn toggle(&self) -> esphomeapi::Result<()> {
    match self.is_on()? {
      true => self.turn_off().await,
      false => self.turn_on().await,
    }
  }

  pub async fn set_state(&self, state: bool) -> esphomeapi::Result<()> {
    match state {
      true => self.turn_on().await,
      false => self.turn_off().await,
    }
  }
}

impl BaseEntity for Switch {
  fn key(&self) -> u32 {
    self.info.entity_info.key
  }

  fn name(&self) -> String {
    self.info.entity_info.name.clone()
  }
}
