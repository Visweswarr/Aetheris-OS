//! Comprehensive tests for the Prompt Router & System Instructions module

use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

use aetheris_ai_core::router::{
    PromptRouter, Intent, TemplateVariables, ConversationTurn, 
    TemplateValidator, RenderContext
};
use aetheris_ai_core::ipc::ChatRequest;

/// Test helper to create a temporary templates directory with sample templates
async fn create_test_templates() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("prompts");
    fs::create_dir_all(&templates_dir).await.unwrap();

    // Create test templates for each intent
    let templates = vec![
        ("chat", "Hello {{user_message}}! Session: {{session_id}}"),
        ("summarize", "Summarize: {{user_message}} at {{timestamp}}"),
        ("command", "Execute: {{user_message}} for user {{user_id}}"),
        ("code", "Code request: {{user_message}} in {{language}}"),
        ("creative", "Creative task: {{user_message}}"),
        ("analysis", "Analyze: {{user_message}}"),
        ("translate", "Translate: {{user_message}} to {{language}}"),
        ("qa", "Question: {{user_message}}"),
    ];

    for (intent, content) in templates {
        let template_path = templates_dir.join(format!("{}.tmpl", intent));
        fs::write(&template_path, content).await.unwrap();
    }

    (temp_dir, templates_dir)
}

/// Test helper to create a sample chat request
fn create_sample_request() -> ChatRequest {
    ChatRequest {
        message: "Hello, how are you?".to_string(),
        conversation_history: None,
        session_id: Some("test-session-123".to_string()),
        user_id: Some("test-user-456".to_string()),
        conversation_id: Some("test-conv-789".to_string()),
        metadata: Some({
            let mut meta = HashMap::new();
            meta.insert("intent".to_string(), "chat".to_string());
            meta.insert("custom_var".to_string(), "test_value".to_string());
            meta
        }),
        language: Some("en".to_string()),
        enable_function_calling: false,
        allowed_functions: Vec::new(),
    }
}

#[tokio::test]
async fn test_router_initialization() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    
    // Initialize the router
    router.initialize().await.unwrap();
    
    // Check that all templates are loaded
    let intents = router.get_available_intents().await;
    assert_eq!(intents.len(), 8);
    
    // Verify specific intents are available
    assert!(intents.contains(&Intent::Chat));
    assert!(intents.contains(&Intent::Summarize));
    assert!(intents.contains(&Intent::Command));
    assert!(intents.contains(&Intent::Code));
    assert!(intents.contains(&Intent::Creative));
    assert!(intents.contains(&Intent::Analysis));
    assert!(intents.contains(&Intent::Translate));
    assert!(intents.contains(&Intent::Qa));
}

#[tokio::test]
async fn test_intent_detection_from_metadata() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    router.initialize().await.unwrap();
    
    let mut request = create_sample_request();
    request.metadata = Some({
        let mut meta = HashMap::new();
        meta.insert("intent".to_string(), "summarize".to_string());
        meta
    });
    
    let intent = router.detect_intent(&request);
    assert_eq!(intent, Intent::Summarize);
}

#[tokio::test]
async fn test_intent_detection_from_content() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    router.initialize().await.unwrap();
    
    // Test various content-based intent detection
    let test_cases = vec![
        ("Please summarize this document", Intent::Summarize),
        ("Write a function to calculate fibonacci", Intent::Code),
        ("Translate this text to Spanish", Intent::Translate),
        ("Analyze the data and explain the trends", Intent::Analysis),
        ("Write a creative story about a robot", Intent::Creative),
        ("Execute the backup command", Intent::Command),
        ("What is the capital of France?", Intent::Qa),
        ("Hello, how are you today?", Intent::Chat),
    ];
    
    for (message, expected_intent) in test_cases {
        let request = ChatRequest {
            message: message.to_string(),
            conversation_history: None,
            session_id: Some("test-session".to_string()),
            user_id: Some("test-user".to_string()),
            conversation_id: Some("test-conv".to_string()),
            metadata: None,
            language: Some("en".to_string()),
            enable_function_calling: false,
            allowed_functions: Vec::new(),
        };
        
        let detected_intent = router.detect_intent(&request);
        assert_eq!(detected_intent, expected_intent, 
                  "Failed for message: '{}'", message);
    }
}

#[tokio::test]
async fn test_template_rendering() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    router.initialize().await.unwrap();
    
    let request = create_sample_request();
    let context = router.route_request(&request).await.unwrap();
    
    // Render the template
    let rendered = router.render_template(&context).await.unwrap();
    
    // Verify the template was rendered correctly
    assert!(rendered.contains("Hello, how are you?"));
    assert!(rendered.contains("test-session-123"));
    assert!(rendered.contains("test-user-456"));
    assert!(rendered.contains("2024-01-01T00:00:00Z")); // deterministic timestamp
}

#[tokio::test]
async fn test_conversation_history_rendering() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    router.initialize().await.unwrap();
    
    let mut request = create_sample_request();
    request.conversation_history = Some(vec![
        aetheris_ai_core::ipc::ConversationTurn {
            role: "user".to_string(),
            content: "Hello there".to_string(),
        },
        aetheris_ai_core::ipc::ConversationTurn {
            role: "assistant".to_string(),
            content: "Hi! How can I help you?".to_string(),
        },
    ]);
    
    let context = router.route_request(&request).await.unwrap();
    let rendered = router.render_template(&context).await.unwrap();
    
    // Verify conversation history is included
    assert!(rendered.contains("user: Hello there"));
    assert!(rendered.contains("assistant: Hi! How can I help you?"));
}

#[tokio::test]
async fn test_custom_variables_rendering() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    router.initialize().await.unwrap();
    
    let mut request = create_sample_request();
    request.metadata = Some({
        let mut meta = HashMap::new();
        meta.insert("custom_var".to_string(), "custom_value".to_string());
        meta.insert("another_var".to_string(), "another_value".to_string());
        meta
    });
    
    let context = router.route_request(&request).await.unwrap();
    let rendered = router.render_template(&context).await.unwrap();
    
    // Verify custom variables are accessible (though not directly in our simple templates)
    // The variables should be available in the context
    assert_eq!(context.variables.custom_vars.get("custom_var"), Some(&"custom_value".to_string()));
    assert_eq!(context.variables.custom_vars.get("another_var"), Some(&"another_value".to_string()));
}

#[tokio::test]
async fn test_deterministic_rendering() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    router.initialize().await.unwrap();
    
    let request = create_sample_request();
    
    // Render the same template multiple times
    let context1 = router.route_request(&request).await.unwrap();
    let rendered1 = router.render_template(&context1).await.unwrap();
    
    let context2 = router.route_request(&request).await.unwrap();
    let rendered2 = router.render_template(&context2).await.unwrap();
    
    // Results should be identical in deterministic mode
    assert_eq!(rendered1, rendered2);
    
    // Timestamp should be deterministic
    assert_eq!(context1.variables.timestamp, "2024-01-01T00:00:00Z");
    assert_eq!(context2.variables.timestamp, "2024-01-01T00:00:00Z");
}

#[tokio::test]
async fn test_non_deterministic_rendering() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, false); // non-deterministic
    router.initialize().await.unwrap();
    
    let request = create_sample_request();
    
    // Render the same template multiple times
    let context1 = router.route_request(&request).await.unwrap();
    let rendered1 = router.render_template(&context1).await.unwrap();
    
    // Small delay to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    let context2 = router.route_request(&request).await.unwrap();
    let rendered2 = router.render_template(&context2).await.unwrap();
    
    // Timestamps should be different in non-deterministic mode
    assert_ne!(context1.variables.timestamp, context2.variables.timestamp);
}

#[tokio::test]
async fn test_template_validation() {
    let template = "Hello {{user_message}}! Your ID is {{user_id}} and the time is {{timestamp}}";
    let required_vars = TemplateValidator::validate_template(template).unwrap();
    
    assert_eq!(required_vars.len(), 3);
    assert!(required_vars.contains(&"user_message".to_string()));
    assert!(required_vars.contains(&"user_id".to_string()));
    assert!(required_vars.contains(&"timestamp".to_string()));
}

#[tokio::test]
async fn test_variable_coverage_check() {
    let template = "{{user_message}} {{user_id}} {{custom_var}}";
    
    let mut variables = TemplateVariables {
        user_message: "Hello".to_string(),
        conversation_history: Vec::new(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        user_id: Some("user123".to_string()),
        session_id: None,
        conversation_id: None,
        language: None,
        model_name: None,
        system_context: None,
        custom_vars: HashMap::new(),
        seed: Some(42),
    };
    
    // Test with missing custom variable
    let missing = TemplateValidator::check_variable_coverage(template, &variables).unwrap();
    assert_eq!(missing.len(), 1);
    assert!(missing.contains(&"custom_var".to_string()));
    
    // Add the missing variable
    variables.custom_vars.insert("custom_var".to_string(), "value".to_string());
    let missing = TemplateValidator::check_variable_coverage(template, &variables).unwrap();
    assert_eq!(missing.len(), 0);
}

#[tokio::test]
async fn test_conversation_history_formatting() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    
    let history = vec![
        ConversationTurn {
            role: "user".to_string(),
            content: "Hello".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
        ConversationTurn {
            role: "assistant".to_string(),
            content: "Hi there!".to_string(),
            timestamp: "2024-01-01T00:00:01Z".to_string(),
        },
        ConversationTurn {
            role: "user".to_string(),
            content: "How are you?".to_string(),
            timestamp: "2024-01-01T00:00:02Z".to_string(),
        },
    ];
    
    let formatted = router.format_conversation_history(&history);
    assert_eq!(formatted, "user: Hello\nassistant: Hi there!\nuser: How are you?");
}

#[tokio::test]
async fn test_empty_conversation_history() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    
    let history = vec![];
    let formatted = router.format_conversation_history(&history);
    assert_eq!(formatted, "No previous conversation.");
}

#[tokio::test]
async fn test_clean_unreplaced_placeholders() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    
    let template = "Hello {{user_message}} {{missing_var}} World {{another_missing}}";
    let cleaned = router.clean_unreplaced_placeholders(template);
    assert_eq!(cleaned, "Hello {{user_message}} [MISSING_VARIABLE] World [MISSING_VARIABLE]");
}

#[tokio::test]
async fn test_router_reload_templates() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir.clone(), true);
    
    // Initial load
    router.initialize().await.unwrap();
    let initial_count = router.get_available_intents().await.len();
    assert_eq!(initial_count, 8);
    
    // Add a new template
    let new_template_path = templates_dir.join("custom.tmpl");
    fs::write(&new_template_path, "Custom template: {{user_message}}").await.unwrap();
    
    // Reload templates
    router.reload_templates().await.unwrap();
    let new_count = router.get_available_intents().await.len();
    assert_eq!(new_count, 9);
    
    // Verify the new intent is available
    let intents = router.get_available_intents().await;
    assert!(intents.contains(&Intent::Custom("custom".to_string())));
}

#[tokio::test]
async fn test_router_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let non_existent_dir = temp_dir.path().join("non_existent");
    let router = PromptRouter::new(non_existent_dir, true);
    
    // Should fail to initialize with non-existent directory
    let result = router.initialize().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_router_missing_template() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    router.initialize().await.unwrap();
    
    let mut request = create_sample_request();
    request.metadata = Some({
        let mut meta = HashMap::new();
        meta.insert("intent".to_string(), "nonexistent".to_string());
        meta
    });
    
    let context = router.route_request(&request).await.unwrap();
    let result = router.render_template(&context).await;
    
    // Should fail to render non-existent template
    assert!(result.is_err());
}

#[tokio::test]
async fn test_intent_to_string_conversion() {
    assert_eq!(Intent::Chat.to_string(), "chat");
    assert_eq!(Intent::Summarize.to_string(), "summarize");
    assert_eq!(Intent::Command.to_string(), "command");
    assert_eq!(Intent::Code.to_string(), "code");
    assert_eq!(Intent::Creative.to_string(), "creative");
    assert_eq!(Intent::Analysis.to_string(), "analysis");
    assert_eq!(Intent::Translate.to_string(), "translate");
    assert_eq!(Intent::Qa.to_string(), "qa");
    assert_eq!(Intent::Custom("test".to_string()).to_string(), "test");
}

#[tokio::test]
async fn test_intent_from_string_conversion() {
    assert_eq!(Intent::from("chat"), Intent::Chat);
    assert_eq!(Intent::from("summarize"), Intent::Summarize);
    assert_eq!(Intent::from("command"), Intent::Command);
    assert_eq!(Intent::from("code"), Intent::Code);
    assert_eq!(Intent::from("creative"), Intent::Creative);
    assert_eq!(Intent::from("analysis"), Intent::Analysis);
    assert_eq!(Intent::from("translate"), Intent::Translate);
    assert_eq!(Intent::from("qa"), Intent::Qa);
    assert_eq!(Intent::from("custom_intent"), Intent::Custom("custom_intent".to_string()));
    
    // Test case insensitivity
    assert_eq!(Intent::from("CHAT"), Intent::Chat);
    assert_eq!(Intent::from("Summarize"), Intent::Summarize);
    assert_eq!(Intent::from("COMMAND"), Intent::Command);
}

#[tokio::test]
async fn test_template_path_generation() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir.clone(), true);
    
    let chat_path = router.get_template_path(&Intent::Chat);
    assert_eq!(chat_path, templates_dir.join("chat.tmpl"));
    
    let custom_path = router.get_template_path(&Intent::Custom("test".to_string()));
    assert_eq!(custom_path, templates_dir.join("test.tmpl"));
}

#[tokio::test]
async fn test_router_has_template() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    router.initialize().await.unwrap();
    
    assert!(router.has_template(&Intent::Chat).await);
    assert!(router.has_template(&Intent::Summarize).await);
    assert!(!router.has_template(&Intent::Custom("nonexistent".to_string())).await);
}

#[tokio::test]
async fn test_complex_template_rendering() {
    let (temp_dir, templates_dir) = create_test_templates().await;
    let router = PromptRouter::new(templates_dir, true);
    router.initialize().await.unwrap();
    
    let mut request = create_sample_request();
    request.message = "Please analyze this data and provide insights".to_string();
    request.conversation_history = Some(vec![
        aetheris_ai_core::ipc::ConversationTurn {
            role: "user".to_string(),
            content: "I have some data to analyze".to_string(),
        },
    ]);
    request.metadata = Some({
        let mut meta = HashMap::new();
        meta.insert("data_type".to_string(), "sales".to_string());
        meta.insert("timeframe".to_string(), "Q1 2024".to_string());
        meta
    });
    
    let context = router.route_request(&request).await.unwrap();
    let rendered = router.render_template(&context).await.unwrap();
    
    // Verify all components are rendered
    assert!(rendered.contains("Please analyze this data and provide insights"));
    assert!(rendered.contains("test-session-123"));
    assert!(rendered.contains("test-user-456"));
    assert!(rendered.contains("user: I have some data to analyze"));
    assert!(rendered.contains("2024-01-01T00:00:00Z"));
    
    // Verify custom variables are available
    assert_eq!(context.variables.custom_vars.get("data_type"), Some(&"sales".to_string()));
    assert_eq!(context.variables.custom_vars.get("timeframe"), Some(&"Q1 2024".to_string()));
}
