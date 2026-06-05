//! Example client for AI Core Service.
//!
//! The production IPC transport is Unix-domain-socket based. Keep this
//! example buildable on Windows by making the non-Unix entry point explicit.

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("AI Core Unix IPC client example placeholder");
    Ok(())
}

#[cfg(not(unix))]
fn main() {
    println!("AI Core Unix IPC client example is only available on Unix platforms.");
}
