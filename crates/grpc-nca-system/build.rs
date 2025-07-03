use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .compile_protos(&["protos/nca_system.proto"], &["protos"])?;
    Ok(())
}
