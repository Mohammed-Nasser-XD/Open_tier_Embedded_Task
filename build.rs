use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {

    // Compile protobuf definitions into Rust code
    prost_build::compile_protos(&["proto/messages.proto"], &["proto/"])?;

    Ok(())
}
