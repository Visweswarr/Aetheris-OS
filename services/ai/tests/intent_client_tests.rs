use std::env;
use std::fs;
use std::path::PathBuf;

#[tokio::test]
async fn writes_v1_envelope_to_file_endpoint() {
    // Arrange: use to-file endpoint to capture bytes
    let dir = PathBuf::from("data/test_intents");
    let _ = fs::create_dir_all(&dir);
    let out = dir.join("intent_out.cborl");
    let _ = fs::remove_file(&out);
    env::set_var("INTENT_SOCKET", format!("to-file://{}", out.display()));
    // No token in test
    env::remove_var("AICORE_CAPTOKEN");

    // Build CBOR params and send
    let params = serde_cbor::value::Value::Map(vec![
        (serde_cbor::Value::Text("action".into()), serde_cbor::Value::Text("notify".into())),
        (serde_cbor::Value::Text("text".into()), serde_cbor::Value::Text("hello".into())),
    ]);
    let resp = services::ai::intent_client::send_intent("system.notification", &params).await;
    assert!(resp.is_ok());

    // Read the written CBOR line and decode EnvelopeV1
    let bytes = fs::read(&out).expect("read out");
    // Only one line appended
    let first_line = bytes.split(|b| *b == b'\n').next().unwrap().to_vec();
    #[derive(serde::Deserialize)]
    struct EnvelopeV1 { v: u8, captoken: Vec<u8>, payload: Vec<u8> }
    let env: EnvelopeV1 = ciborium::de::from_reader(first_line.as_slice()).expect("decode env");
    assert_eq!(env.v, 1);
    assert!(env.captoken.is_empty());
    // Decode inner payload
    let inner: serde_cbor::Value = ciborium::de::from_reader(env.payload.as_slice()).expect("decode payload");
    if let serde_cbor::Value::Map(m) = inner { assert!(m.len() >= 1); } else { panic!("map expected"); }
}
