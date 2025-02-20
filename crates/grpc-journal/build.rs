use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .compile_protos(&["protos/api.proto"], &["protos"])?;
    Ok(())
}
