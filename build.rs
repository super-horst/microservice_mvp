extern crate tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &["proto/event_svc.proto"],
            &["proto"],
        )?;


    Ok(())
}
