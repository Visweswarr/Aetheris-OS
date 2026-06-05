use serde::{Deserialize, Serialize};
use std::io::{self};
use policy_engine::cap::CapVerifier;
use policy_engine::audit::{AuditWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnvelopeV1 {
    v: u8,
    captoken: Vec<u8>,
    payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IntentResponse {
    status: String,
    preview_id: Option<String>,
    commit_id: Option<String>,
    message: Option<String>,
}

fn compute_preview_id(token: &[u8], payload: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(token);
    hasher.update(payload);
    format!("{:x}", hasher.finalize())
}

#[cfg(target_family = "unix")]
async fn run_unix(path: &str) -> std::io::Result<()> {
    use tokio::net::UnixListener;
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;
    let _ = std::fs::remove_file(path);
    let listener = UnixListener::bind(path)?;
    println!("listening on unix://{}", path);
    let verifier = CapVerifier::from_env().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let audit = AuditWriter::from_env();
    loop {
        let (mut stream, _) = listener.accept().await?;
        let verifier = verifier.clone();
        let audit = AuditWriter::from_env();
        let endpoint = format!("unix://{}", path);
        tokio::spawn(async move {
            let res = async {
                let mut lbuf = [0u8; 4];
                stream.read_exact(&mut lbuf).await?;
                let len = u32::from_be_bytes(lbuf) as usize;
                let mut buf = vec![0u8; len];
                stream.read_exact(&mut buf).await?;
                let env: EnvelopeV1 = serde_cbor::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                let mut resp = IntentResponse { status: "deny".into(), preview_id: None, commit_id: None, message: None };
                match verifier.verify(&env.captoken) {
                    Ok(claims) => {
                        if CapVerifier::require(&claims.scopes, "intent.emit") {
                            let preview = compute_preview_id(&env.captoken, &env.payload);
                            resp.status = "ok".into();
                            resp.preview_id = Some(preview);
                            let _ = audit.write(&AuditWriter::allow("intent.preview", &claims.iss, &claims.sub, &claims.aud, &claims.jti_hex, &claims.scopes, Some(endpoint.clone()), AuditWriter::now_ts90k()));
                        } else {
                            resp.message = Some("missing scope".into());
                            let _ = audit.write(&AuditWriter::deny("intent.preview", &claims.iss, &claims.sub, &claims.aud, &claims.jti_hex, &claims.scopes, "missing scope", Some(endpoint.clone()), AuditWriter::now_ts90k()));
                        }
                    }
                    Err(e) => {
                        resp.message = Some("invalid token".into());
                        let _ = audit.write(&AuditWriter::deny("intent.preview", "?", "?", "?", "", &Vec::new(), &e.to_string(), Some(endpoint.clone()), AuditWriter::now_ts90k()));
                    }
                }
                let mut out = Vec::new();
                serde_cbor::to_writer(&mut out, &resp).unwrap();
                let blen = (out.len() as u32).to_be_bytes();
                stream.write_all(&blen).await?;
                stream.write_all(&out).await?;
                stream.flush().await?;
                Ok::<(), io::Error>(())
            }.await;
            if let Err(e) = res { eprintln!("unix handler error: {}", e); }
        });
    }
}

#[cfg(target_os = "windows")]
async fn run_pipe(name: &str) -> std::io::Result<()> {
    use named_pipe::{PipeOptions, PipeServer};
    println!("listening on pipe://{}", name);
    let verifier = CapVerifier::from_env().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let audit = AuditWriter::from_env();
    loop {
        let server: PipeServer = PipeOptions::new(name).single()?
            .expect("failed to create pipe server");
        let verifier = verifier.clone();
        let audit = AuditWriter::from_env();
        let endpoint = format!("pipe://{}", name);
        tokio::task::spawn_blocking(move || {
            let mut stream = server;
            if let Err(e) = handle_stream_pipe(&mut stream, &verifier, &audit, &endpoint) {
                eprintln!("pipe handler error: {}", e);
            }
        });
    }
}

#[cfg(target_os = "windows")]
fn handle_stream_pipe(stream: &mut named_pipe::PipeServer, verifier: &CapVerifier, audit: &AuditWriter, endpoint: &str) -> std::io::Result<()> {
    use std::io::{Read, Write};
    let mut lbuf = [0u8; 4];
    stream.read_exact(&mut lbuf)?;
    let len = u32::from_be_bytes(lbuf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    let env: EnvelopeV1 = serde_cbor::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let mut resp = IntentResponse { status: "deny".into(), preview_id: None, commit_id: None, message: None };
    match verifier.verify(&env.captoken) {
        Ok(claims) => {
            if CapVerifier::require(&claims.scopes, "intent.emit") {
                let preview = compute_preview_id(&env.captoken, &env.payload);
                resp.status = "ok".into();
                resp.preview_id = Some(preview);
                let _ = audit.write(&AuditWriter::allow("intent.preview", &claims.iss, &claims.sub, &claims.aud, &claims.jti_hex, &claims.scopes, Some(endpoint.to_string()), AuditWriter::now_ts90k()));
            } else {
                resp.message = Some("missing scope".into());
                let _ = audit.write(&AuditWriter::deny("intent.preview", &claims.iss, &claims.sub, &claims.aud, &claims.jti_hex, &claims.scopes, "missing scope", Some(endpoint.to_string()), AuditWriter::now_ts90k()));
            }
        }
        Err(e) => {
            resp.message = Some("invalid token".into());
            let _ = audit.write(&AuditWriter::deny("intent.preview", "?", "?", "?", "", &Vec::new(), &e.to_string(), Some(endpoint.to_string()), AuditWriter::now_ts90k()));
        }
    }
    let mut out = Vec::new();
    serde_cbor::to_writer(&mut out, &resp).unwrap();
    let blen = (out.len() as u32).to_be_bytes();
    stream.write_all(&blen)?;
    stream.write_all(&out)?;
    stream.flush()?;
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 { 
        eprintln!("Usage: intent_echo_server --unix /tmp/aetheris-intent.sock | --pipe \\./\\pipe\\aetheris-intent");
        std::process::exit(2);
    }
    match args[1].as_str() {
        "--unix" => {
            #[cfg(target_family = "unix")]
            {
                run_unix(&args[2]).await
            }
            #[cfg(not(target_family = "unix"))]
            { 
                eprintln!("unix sockets not supported on this platform");
                std::process::exit(3);
            }
        }
        "--pipe" => {
            #[cfg(target_os = "windows")]
            {
                run_pipe(&args[2]).await
            }
            #[cfg(not(target_os = "windows"))]
            {
                eprintln!("named pipes not supported on this platform");
                std::process::exit(3);
            }
        }
        _ => {
            eprintln!("Unknown flag: {}", args[1]);
            std::process::exit(2);
        }
    }
}
                run_pipe(&args[2]).await
            }
            #[cfg(not(target_os = "windows"))]
            {
                eprintln!("named pipes not supported on this platform");
                std::process::exit(3);
            }
        }
        _ => {
            eprintln!("Unknown flag: {}", args[1]);
            std::process::exit(2);
        }
    }
}
