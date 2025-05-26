mod switch;

use napi_derive::napi;
pub use switch::Switch;

#[napi]
pub enum Entity {
  Switch(u32),
  Sensor(u32),
}
