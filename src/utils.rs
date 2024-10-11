use protobuf::{Message, MessageDyn, MessageFull};

use crate::proto::api_options::exts::id;

pub trait Options {
  fn get_option_id() -> u32;
  fn create_message_with_type() -> (u32, Self);
}

impl<T> Options for T
where
  T: MessageFull + MessageDyn + Message,
{
  fn get_option_id() -> u32 {
    let msg = T::descriptor();
    let options = msg.proto().options.as_ref().unwrap();
    id.get(options).unwrap()
  }

  fn create_message_with_type() -> (u32, Self) {
    let msg = T::descriptor();
    let options = msg.proto().options.as_ref().unwrap();
    let option_id = id.get(options).unwrap();
    (option_id, T::default())
  }
}
