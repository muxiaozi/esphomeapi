use protobuf::{Message, MessageDyn, MessageFull};

use crate::proto::api_options::exts::id;

use super::codec::DynamicResponseMessage;

pub trait Options {
  fn get_option_id() -> u32;
  fn create_response_message() -> DynamicResponseMessage;
}

impl<T> Options for T where T: Message + MessageFull + MessageDyn {
  fn get_option_id() -> u32 {
    let msg = T::descriptor();
    let options = msg.proto().options.as_ref().unwrap();
    id.get(options).unwrap()
  }

  fn create_response_message() -> DynamicResponseMessage {
    (Self::get_option_id(), Box::new(T::default()))
  }
}