use std::io::Result;

fn main() -> Result<()> {
    // prost_build::compile_protos(&["src/proto/api_options.proto", "src/proto/api.proto"], &["src/proto"])?;
    protobuf_codegen::Codegen::new()
        .protoc()
        .includes(&["src/protos"])
        .input("src/protos/api_options.proto")
        .input("src/protos/api.proto")
        .cargo_out_dir("protos")
        .run()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}