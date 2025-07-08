mod light;
mod switch;

use napi_derive::napi;

pub use light::Light;
pub use switch::Switch;

#[napi]
pub enum Entity {
  Light(u32),
  Switch(u32),
  Sensor(u32),
}
