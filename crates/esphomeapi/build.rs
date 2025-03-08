use std::{fs, io::Result, path::Path};

use protobuf::{
  descriptor::field_descriptor_proto::Type,
  reflect::{FieldDescriptor, MessageDescriptor},
};
use protobuf_codegen::{Customize, CustomizeCallback};

fn main() -> Result<()> {
  struct GenNapi;

  impl CustomizeCallback for GenNapi {
    fn message(&self, _message: &MessageDescriptor) -> Customize {
      Customize::default().before("#[napi(object)]")
    }

    fn field(&self, field: &FieldDescriptor) -> Customize {
      if field.proto().type_() == Type::TYPE_ENUM {
        Customize::default().before("#[napi]")
      } else {
        Customize::default()
      }
    }

    fn special_field(&self, _message: &MessageDescriptor, _field: &str) -> Customize {
      Customize::default().before("#[napi(skip)]")
    }
  }

  protobuf_codegen::Codegen::new()
    .protoc()
    .includes(&["src/protos"])
    .input("src/protos/api_options.proto")
    .input("src/protos/api.proto")
    .customize_callback(GenNapi)
    .cargo_out_dir("protos")
    .run_from_script();

  // Post-process the generated files to add the import at the top
  let out_dir = std::env::var("OUT_DIR").unwrap();
  let protos_dir = Path::new(&out_dir).join("protos");

  for entry in fs::read_dir(&protos_dir).unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();

    if path.extension().unwrap_or_default() == "rs" {
      // Read the generated file
      let content = fs::read_to_string(&path).unwrap();

      // Add the import at the top of the file
      let new_content = format!("use napi_derive::napi;\n\n{}", content);

      // Write back to the file
      fs::write(&path, new_content).unwrap();
    }
  }

  Ok(())
}
