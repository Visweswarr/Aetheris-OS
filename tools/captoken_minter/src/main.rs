use clap::Parser;
use coset::{CoseSign1, HeaderBuilder, iana::Algorithm};
use ed25519_dalek::{SigningKey, Signer, SecretKey, VerifyingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)] iss: String,
    #[arg(long)] sub: String,
    #[arg(long)] aud: String,
    #[arg(long)] exp: u64,
    #[arg(long)] scopes: String,
    #[arg(long)] out: String,
    #[arg(long)] key: String,
    #[arg(long, default_value_t=String::from("dev-1"))] kid: String,
    #[arg(long)] out_jwks: Option<String>,
}

#[derive(Serialize)]
struct Claims { iss:String, sub:String, aud:String, iat:u64, nbf:u64, exp:u64, jti:Vec<u8>, scopes: Vec<String> }

fn main() {
    let args = Args::parse();
    let sk_bytes = fs::read(&args.key).expect("read sk");
    let sk = SecretKey::from_bytes(&sk_bytes[..32]).expect("sk");
    let signing = SigningKey::from_bytes(&sk);
    let vk: VerifyingKey = signing.verifying_key();
    // claims
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut jti = vec![0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut jti);
    let claims = Claims { iss: args.iss.clone(), sub: args.sub.clone(), aud: args.aud.clone(), iat: now, nbf: now, exp: now + args.exp, jti: jti.clone(), scopes: args.scopes.split(',').map(|s| s.to_string()).collect() };
    let mut payload = Vec::new();
    ciborium::ser::into_writer(&claims, &mut payload).unwrap();
    let protected = HeaderBuilder::new().alg(Algorithm::EdDSA).kid(args.kid.as_bytes().to_vec()).build();
    let mut sign1 = CoseSign1 { protected, unprotected: Default::default(), payload: Some(payload), signature: Default::default() };
    sign1.sign(|m| signing.sign(m).to_bytes().to_vec()).unwrap();
    let token = sign1.to_vec().unwrap();
    fs::write(&args.out, &token).expect("write token");
    println!("Wrote token to {} (jti={})", args.out, hex::encode(jti));
    if let Some(out_jwks) = args.out_jwks {
        let x_b64 = URL_SAFE_NO_PAD.encode(vk.to_bytes());
        let jwks = serde_json::json!({
            "keys": [ {"kty":"OKP","crv":"Ed25519","kid": args.kid, "x": x_b64 } ]
        });
        fs::write(out_jwks, serde_json::to_vec_pretty(&jwks).unwrap()).expect("write jwks");
    }
}
