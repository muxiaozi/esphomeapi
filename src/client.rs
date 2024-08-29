use tokio::net::ToSocketAddrs;

use crate::connection::Connection;

pub struct Client {
  connection: Connection,
}
