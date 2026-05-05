fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &["../../proto/bot_messaging.proto", "../../proto/data_service.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
