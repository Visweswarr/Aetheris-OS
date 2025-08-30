fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile protobuf definitions
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &["../../proto/dtn.proto"],
            &["../../proto"],
        )?;
    
    // Re-run if protobuf files change
    println!("cargo:rerun-if-changed=../../proto/dtn.proto");
    
    Ok(())
}
