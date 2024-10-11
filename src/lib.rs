mod proto {
  include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
  // include!(concat!(env!("OUT_DIR"), "/protos.rs"));
}
pub use proto::api;

mod client;
mod connection;
mod model;
mod utils;

pub use client::Client;
pub use connection::Connection;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
