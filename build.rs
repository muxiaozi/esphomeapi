use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/proto/api_options.proto", "src/proto/api.proto"], &["src/proto"])?;
    Ok(())
}