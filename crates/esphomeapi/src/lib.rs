mod proto {
  include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
}

pub use proto::api;

mod client;
mod connection;
pub mod model;
mod utils;

pub use client::Client;
pub use connection::Connection;
pub use utils::Options;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
