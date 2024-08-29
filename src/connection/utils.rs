use protobuf::{Message, MessageFull};

use crate::proto::api_options::exts::id;

pub trait Options {
  fn get_option_id() -> Option<u32>;
}

impl<T> Options for T where T: Message + MessageFull {
  fn get_option_id() -> Option<u32> {
    let msg = T::descriptor();
    let options = msg.proto().options.as_ref();
    options.and_then(|options| id.get(options))
  }
}