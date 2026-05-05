fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(&["../../proto/data_service.proto"], &["../../proto"])?;
    Ok(())
}
