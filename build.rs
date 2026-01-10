use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "protos/common.proto",
                "protos/browser.proto",
                "protos/page.proto",
                "protos/element.proto",
                "protos/profile.proto",
                "protos/event.proto",
            ],
            &["protos"],
        )?;
    Ok(())
}
