mod proto {
  include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

mod connection;
pub use connection::Connection;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
