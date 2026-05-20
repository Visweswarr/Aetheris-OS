use std::fs;

use wasm_driver_host::host::Host;

#[tokio::test(flavor = "multi_thread")]
async fn host_loads_component_model_binary() {
    let host = Host::new().expect("host init");
    let dir = tempfile::tempdir().expect("tempdir");
    let component_path = dir.path().join("hello_component.wasm");
    let component_bytes = wat::parse_str(r#"(component)"#).expect("component wat");
    fs::write(&component_path, component_bytes).expect("write component");

    let component = host
        .load_component(&component_path)
        .expect("load component");
    let (_store, _instance) = host
        .instantiate(&component)
        .await
        .expect("instantiate component");
}
