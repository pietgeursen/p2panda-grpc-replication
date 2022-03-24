use doggo_build::prost::Builder;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Builder::default()
        .compile(&["proto/replication.proto"], &["proto/"])?;
    Ok(())
}
