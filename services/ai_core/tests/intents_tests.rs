//! Comprehensive tests for the System Intents module

use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use serde_json::Value;

use aetheris_ai_core::intents::{
    SystemIntentManager, SystemIntent, SystemActionResult, SystemActionContext,
    SystemActionAdapter, create_system_intent_manager
};
use aetheris_ai_core::cap::CapTokenManager;

/// Test helper to create a system intent manager with mock adapter
async fn create_test_intent_manager() -> SystemIntentManager {
    let temp_dir = TempDir::new().unwrap();
    let cap_token_manager = Arc::new(
        CapTokenManager::new(&temp_dir.path().join("cap.toml")).await.unwrap()
    );
    let adapter = Arc::new(aetheris_ai_core::intents::mock::MockSystemAdapter::new());
    SystemIntentManager::new(adapter, cap_token_manager)
}

/// Test helper to create a system action context
fn create_test_context() -> SystemActionContext {
    SystemActionContext {
        user_id: "test_user".to_string(),
        session_id: "test_session".to_string(),
        cap_token: Some("test_cap_token".to_string()),
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_intent_parsing_settings() {
    let manager = create_test_intent_manager().await;
    
    // Test basic settings intent
    let intent = manager.parse_intent("Open Settings").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettings);
    
    // Test settings with category
    let intent = manager.parse_intent("Open display settings").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettingsCategory("display".to_string()));
    
    let intent = manager.parse_intent("Open sound settings").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettingsCategory("sound".to_string()));
    
    let intent = manager.parse_intent("Open network settings").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettingsCategory("network".to_string()));
}

#[tokio::test]
async fn test_intent_parsing_bluetooth() {
    let manager = create_test_intent_manager().await;
    
    // Test enable Bluetooth
    let intent = manager.parse_intent("Turn on Bluetooth").unwrap();
    assert_eq!(intent, SystemIntent::ToggleBluetooth(true));
    
    let intent = manager.parse_intent("Enable Bluetooth").unwrap();
    assert_eq!(intent, SystemIntent::ToggleBluetooth(true));
    
    // Test disable Bluetooth
    let intent = manager.parse_intent("Turn off Bluetooth").unwrap();
    assert_eq!(intent, SystemIntent::ToggleBluetooth(false));
    
    let intent = manager.parse_intent("Disable Bluetooth").unwrap();
    assert_eq!(intent, SystemIntent::ToggleBluetooth(false));
}

#[tokio::test]
async fn test_intent_parsing_wifi() {
    let manager = create_test_intent_manager().await;
    
    // Test enable WiFi
    let intent = manager.parse_intent("Turn on WiFi").unwrap();
    assert_eq!(intent, SystemIntent::ToggleWifi(true));
    
    let intent = manager.parse_intent("Enable Wi-Fi").unwrap();
    assert_eq!(intent, SystemIntent::ToggleWifi(true));
    
    // Test disable WiFi
    let intent = manager.parse_intent("Turn off WiFi").unwrap();
    assert_eq!(intent, SystemIntent::ToggleWifi(false));
    
    let intent = manager.parse_intent("Disable Wi-Fi").unwrap();
    assert_eq!(intent, SystemIntent::ToggleWifi(false));
}

#[tokio::test]
async fn test_intent_parsing_airplane_mode() {
    let manager = create_test_intent_manager().await;
    
    // Test enable airplane mode
    let intent = manager.parse_intent("Turn on airplane mode").unwrap();
    assert_eq!(intent, SystemIntent::ToggleAirplaneMode(true));
    
    let intent = manager.parse_intent("Enable flight mode").unwrap();
    assert_eq!(intent, SystemIntent::ToggleAirplaneMode(true));
    
    // Test disable airplane mode
    let intent = manager.parse_intent("Turn off airplane mode").unwrap();
    assert_eq!(intent, SystemIntent::ToggleAirplaneMode(false));
    
    let intent = manager.parse_intent("Disable flight mode").unwrap();
    assert_eq!(intent, SystemIntent::ToggleAirplaneMode(false));
}

#[tokio::test]
async fn test_intent_parsing_volume() {
    let manager = create_test_intent_manager().await;
    
    // Test volume with percentage
    let intent = manager.parse_intent("Set volume to 50%").unwrap();
    assert_eq!(intent, SystemIntent::AdjustVolume(0.5));
    
    let intent = manager.parse_intent("Set volume to 75%").unwrap();
    assert_eq!(intent, SystemIntent::AdjustVolume(0.75));
    
    // Test volume with decimal
    let intent = manager.parse_intent("Set volume to 0.8").unwrap();
    assert_eq!(intent, SystemIntent::AdjustVolume(0.8));
    
    // Test volume adjustments
    let intent = manager.parse_intent("Turn volume up").unwrap();
    assert_eq!(intent, SystemIntent::AdjustVolume(0.1));
    
    let intent = manager.parse_intent("Turn volume down").unwrap();
    assert_eq!(intent, SystemIntent::AdjustVolume(-0.1));
}

#[tokio::test]
async fn test_intent_parsing_brightness() {
    let manager = create_test_intent_manager().await;
    
    // Test brightness with percentage
    let intent = manager.parse_intent("Set brightness to 80%").unwrap();
    assert_eq!(intent, SystemIntent::SetBrightness(0.8));
    
    let intent = manager.parse_intent("Set brightness to 100%").unwrap();
    assert_eq!(intent, SystemIntent::SetBrightness(1.0));
    
    // Test brightness with decimal
    let intent = manager.parse_intent("Set brightness to 0.6").unwrap();
    assert_eq!(intent, SystemIntent::SetBrightness(0.6));
    
    // Test brightness adjustments
    let intent = manager.parse_intent("Turn brightness up").unwrap();
    assert_eq!(intent, SystemIntent::SetBrightness(0.1));
    
    let intent = manager.parse_intent("Turn brightness down").unwrap();
    assert_eq!(intent, SystemIntent::SetBrightness(-0.1));
}

#[tokio::test]
async fn test_intent_parsing_notes() {
    let manager = create_test_intent_manager().await;
    
    let intent = manager.parse_intent("Create a new note").unwrap();
    assert_eq!(intent, SystemIntent::CreateNote);
    
    let intent = manager.parse_intent("New note").unwrap();
    assert_eq!(intent, SystemIntent::CreateNote);
    
    let intent = manager.parse_intent("Create note").unwrap();
    assert_eq!(intent, SystemIntent::CreateNote);
}

#[tokio::test]
async fn test_intent_parsing_file_manager() {
    let manager = create_test_intent_manager().await;
    
    let intent = manager.parse_intent("Open file manager").unwrap();
    assert_eq!(intent, SystemIntent::OpenFileManager);
    
    let intent = manager.parse_intent("Open files").unwrap();
    assert_eq!(intent, SystemIntent::OpenFileManager);
    
    let intent = manager.parse_intent("Open explorer").unwrap();
    assert_eq!(intent, SystemIntent::OpenFileManager);
}

#[tokio::test]
async fn test_intent_parsing_browser() {
    let manager = create_test_intent_manager().await;
    
    let intent = manager.parse_intent("Open browser").unwrap();
    assert_eq!(intent, SystemIntent::OpenBrowser);
    
    let intent = manager.parse_intent("Open web").unwrap();
    assert_eq!(intent, SystemIntent::OpenBrowser);
    
    let intent = manager.parse_intent("Open internet").unwrap();
    assert_eq!(intent, SystemIntent::OpenBrowser);
}

#[tokio::test]
async fn test_intent_parsing_terminal() {
    let manager = create_test_intent_manager().await;
    
    let intent = manager.parse_intent("Open terminal").unwrap();
    assert_eq!(intent, SystemIntent::OpenTerminal);
    
    let intent = manager.parse_intent("Open command").unwrap();
    assert_eq!(intent, SystemIntent::OpenTerminal);
    
    let intent = manager.parse_intent("Open cmd").unwrap();
    assert_eq!(intent, SystemIntent::OpenTerminal);
}

#[tokio::test]
async fn test_intent_parsing_screen_lock() {
    let manager = create_test_intent_manager().await;
    
    let intent = manager.parse_intent("Lock screen").unwrap();
    assert_eq!(intent, SystemIntent::LockScreen);
    
    let intent = manager.parse_intent("Lock").unwrap();
    assert_eq!(intent, SystemIntent::LockScreen);
}

#[tokio::test]
async fn test_intent_parsing_power_management() {
    let manager = create_test_intent_manager().await;
    
    // Test shutdown
    let intent = manager.parse_intent("Shutdown").unwrap();
    assert_eq!(intent, SystemIntent::Shutdown);
    
    let intent = manager.parse_intent("Shut down").unwrap();
    assert_eq!(intent, SystemIntent::Shutdown);
    
    // Test restart
    let intent = manager.parse_intent("Restart").unwrap();
    assert_eq!(intent, SystemIntent::Restart);
    
    let intent = manager.parse_intent("Reboot").unwrap();
    assert_eq!(intent, SystemIntent::Restart);
    
    // Test sleep
    let intent = manager.parse_intent("Sleep").unwrap();
    assert_eq!(intent, SystemIntent::Sleep);
    
    let intent = manager.parse_intent("Suspend").unwrap();
    assert_eq!(intent, SystemIntent::Sleep);
}

#[tokio::test]
async fn test_intent_parsing_custom() {
    let manager = create_test_intent_manager().await;
    
    let intent = manager.parse_intent("Some unknown command").unwrap();
    match intent {
        SystemIntent::Custom(name, params) => {
            assert_eq!(name, "unknown");
            assert!(params.contains_key("original_text"));
            assert_eq!(params["original_text"], Value::String("Some unknown command".to_string()));
        }
        _ => panic!("Expected custom intent"),
    }
}

#[tokio::test]
async fn test_intent_execution_success() {
    let manager = create_test_intent_manager().await;
    let context = create_test_context();
    
    let intent = SystemIntent::OpenSettings;
    let result = manager.execute_intent(&intent, &context).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.intent, intent);
    assert!(result.message.contains("Mock:"));
    assert!(result.platform_data.is_some());
    assert!(result.error.is_none());
    assert!(result.execution_time_ms > 0);
}

#[tokio::test]
async fn test_intent_execution_with_parameters() {
    let manager = create_test_intent_manager().await;
    let context = create_test_context();
    
    let intent = SystemIntent::ToggleBluetooth(true);
    let result = manager.execute_intent(&intent, &context).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.intent, intent);
    assert!(result.message.contains("Bluetooth enabled"));
}

#[tokio::test]
async fn test_intent_execution_volume() {
    let manager = create_test_intent_manager().await;
    let context = create_test_context();
    
    let intent = SystemIntent::AdjustVolume(0.75);
    let result = manager.execute_intent(&intent, &context).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.intent, intent);
    assert!(result.message.contains("75%"));
}

#[tokio::test]
async fn test_intent_execution_brightness() {
    let manager = create_test_intent_manager().await;
    let context = create_test_context();
    
    let intent = SystemIntent::SetBrightness(0.8);
    let result = manager.execute_intent(&intent, &context).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.intent, intent);
    assert!(result.message.contains("80%"));
}

#[tokio::test]
async fn test_intent_execution_custom() {
    let manager = create_test_intent_manager().await;
    let context = create_test_context();
    
    let mut params = HashMap::new();
    params.insert("test_param".to_string(), Value::String("test_value".to_string()));
    let intent = SystemIntent::Custom("test_intent".to_string(), params);
    
    let result = manager.execute_intent(&intent, &context).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.intent, intent);
    assert!(result.message.contains("Custom intent 'test_intent'"));
}

#[tokio::test]
async fn test_capability_checking_with_token() {
    let manager = create_test_intent_manager().await;
    let context = create_test_context();
    
    let intent = SystemIntent::OpenSettings;
    let result = manager.execute_intent(&intent, &context).await;
    
    // Should succeed with capability token
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_capability_checking_without_token() {
    let manager = create_test_intent_manager().await;
    let mut context = create_test_context();
    context.cap_token = None; // Remove capability token
    
    let intent = SystemIntent::OpenSettings;
    let result = manager.execute_intent(&intent, &context).await;
    
    // Should fail without capability token
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Capability token required"));
}

#[tokio::test]
async fn test_platform_name() {
    let manager = create_test_intent_manager().await;
    assert_eq!(manager.platform_name(), "mock");
}

#[tokio::test]
async fn test_supported_intents() {
    let manager = create_test_intent_manager().await;
    let supported = manager.get_supported_intents();
    
    assert!(!supported.is_empty());
    assert!(supported.contains(&SystemIntent::OpenSettings));
    assert!(supported.contains(&SystemIntent::ToggleBluetooth(false)));
    assert!(supported.contains(&SystemIntent::OpenFileManager));
}

#[tokio::test]
async fn test_required_capabilities() {
    let manager = create_test_intent_manager().await;
    
    // Test settings intent capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::OpenSettings);
    assert!(caps.contains(&"settings.read".to_string()));
    
    // Test Bluetooth intent capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::ToggleBluetooth(false));
    assert!(caps.contains(&"hardware.bluetooth.modify".to_string()));
    
    // Test WiFi intent capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::ToggleWifi(false));
    assert!(caps.contains(&"hardware.wifi.modify".to_string()));
    
    // Test volume intent capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::AdjustVolume(0.0));
    assert!(caps.contains(&"hardware.audio.modify".to_string()));
    
    // Test brightness intent capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::SetBrightness(0.0));
    assert!(caps.contains(&"hardware.display.modify".to_string()));
    
    // Test note creation capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::CreateNote);
    assert!(caps.contains(&"filesystem.write".to_string()));
    
    // Test file manager capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::OpenFileManager);
    assert!(caps.contains(&"filesystem.read".to_string()));
    
    // Test browser capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::OpenBrowser);
    assert!(caps.contains(&"network.access".to_string()));
    
    // Test terminal capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::OpenTerminal);
    assert!(caps.contains(&"system.shell.execute".to_string()));
    
    // Test screen lock capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::LockScreen);
    assert!(caps.contains(&"system.session.control".to_string()));
    
    // Test power management capabilities
    let caps = manager.get_required_capabilities(&SystemIntent::Shutdown);
    assert!(caps.contains(&"system.power.shutdown".to_string()));
    
    let caps = manager.get_required_capabilities(&SystemIntent::Restart);
    assert!(caps.contains(&"system.power.restart".to_string()));
    
    let caps = manager.get_required_capabilities(&SystemIntent::Sleep);
    assert!(caps.contains(&"system.power.sleep".to_string()));
}

#[tokio::test]
async fn test_parameterized_intent_capabilities() {
    let manager = create_test_intent_manager().await;
    
    // Test parameterized intents
    let caps = manager.get_required_capabilities(&SystemIntent::OpenSettingsCategory("display".to_string()));
    assert!(caps.contains(&"settings.read".to_string()));
    
    let caps = manager.get_required_capabilities(&SystemIntent::ToggleBluetooth(true));
    assert!(caps.contains(&"hardware.bluetooth.modify".to_string()));
    
    let caps = manager.get_required_capabilities(&SystemIntent::ToggleWifi(false));
    assert!(caps.contains(&"hardware.wifi.modify".to_string()));
    
    let caps = manager.get_required_capabilities(&SystemIntent::AdjustVolume(0.5));
    assert!(caps.contains(&"hardware.audio.modify".to_string()));
    
    let caps = manager.get_required_capabilities(&SystemIntent::SetBrightness(0.8));
    assert!(caps.contains(&"hardware.display.modify".to_string()));
}

#[tokio::test]
async fn test_custom_intent_capabilities() {
    let manager = create_test_intent_manager().await;
    
    let mut params = HashMap::new();
    params.insert("test".to_string(), Value::String("value".to_string()));
    let custom_intent = SystemIntent::Custom("test".to_string(), params);
    
    let caps = manager.get_required_capabilities(&custom_intent);
    assert!(caps.is_empty()); // Custom intents have no predefined capabilities
}

#[tokio::test]
async fn test_intent_execution_timing() {
    let manager = create_test_intent_manager().await;
    let context = create_test_context();
    
    let start_time = std::time::Instant::now();
    let intent = SystemIntent::OpenSettings;
    let result = manager.execute_intent(&intent, &context).await.unwrap();
    let elapsed = start_time.elapsed();
    
    assert!(result.success);
    assert!(result.execution_time_ms > 0);
    assert!(result.execution_time_ms <= elapsed.as_millis() as u64 + 100); // Allow some tolerance
}

#[tokio::test]
async fn test_intent_execution_platform_data() {
    let manager = create_test_intent_manager().await;
    let context = create_test_context();
    
    let intent = SystemIntent::OpenSettings;
    let result = manager.execute_intent(&intent, &context).await.unwrap();
    
    assert!(result.platform_data.is_some());
    let platform_data = result.platform_data.unwrap();
    assert_eq!(platform_data["platform"], "mock");
    assert_eq!(platform_data["mock"], true);
    assert!(platform_data["timestamp"].is_number());
}

#[tokio::test]
async fn test_intent_execution_context_metadata() {
    let manager = create_test_intent_manager().await;
    let mut context = create_test_context();
    
    // Add some metadata
    context.metadata.insert("test_key".to_string(), Value::String("test_value".to_string()));
    context.metadata.insert("test_number".to_string(), Value::Number(42.into()));
    
    let intent = SystemIntent::OpenSettings;
    let result = manager.execute_intent(&intent, &context).await.unwrap();
    
    assert!(result.success);
    // The mock adapter doesn't use context metadata, but the context is passed through
}

#[tokio::test]
async fn test_intent_execution_error_handling() {
    let manager = create_test_intent_manager().await;
    let context = create_test_context();
    
    // Test with unsupported intent (this should not happen with mock adapter, but test the path)
    let mut params = HashMap::new();
    params.insert("unsupported".to_string(), Value::String("true".to_string()));
    let unsupported_intent = SystemIntent::Custom("unsupported".to_string(), params);
    
    // Mock adapter supports all intents, so this should succeed
    let result = manager.execute_intent(&unsupported_intent, &context).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_intent_parsing_case_insensitive() {
    let manager = create_test_intent_manager().await;
    
    // Test case insensitive parsing
    let intent = manager.parse_intent("OPEN SETTINGS").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettings);
    
    let intent = manager.parse_intent("open settings").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettings);
    
    let intent = manager.parse_intent("Open Settings").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettings);
    
    let intent = manager.parse_intent("oPeN sEtTiNgS").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettings);
}

#[tokio::test]
async fn test_intent_parsing_whitespace_handling() {
    let manager = create_test_intent_manager().await;
    
    // Test whitespace handling
    let intent = manager.parse_intent("  Open Settings  ").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettings);
    
    let intent = manager.parse_intent("\tOpen Settings\t").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettings);
    
    let intent = manager.parse_intent("\nOpen Settings\n").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettings);
}

#[tokio::test]
async fn test_intent_parsing_complex_phrases() {
    let manager = create_test_intent_manager().await;
    
    // Test complex phrases
    let intent = manager.parse_intent("Please open the system settings").unwrap();
    assert_eq!(intent, SystemIntent::OpenSettings);
    
    let intent = manager.parse_intent("Can you turn on Bluetooth for me").unwrap();
    assert_eq!(intent, SystemIntent::ToggleBluetooth(true));
    
    let intent = manager.parse_intent("I need to create a new note").unwrap();
    assert_eq!(intent, SystemIntent::CreateNote);
    
    let intent = manager.parse_intent("Could you please lock the screen").unwrap();
    assert_eq!(intent, SystemIntent::LockScreen);
}
