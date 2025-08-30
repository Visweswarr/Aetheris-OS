use kernel::llm::schema::{
    PromptV1, MessageV1, RedactionRuleV1, AdapterConfigV1, QuotaLimitsV1,
    ROLE_USER, BACKEND_NULL, apply_redactions
};

#[test]
fn test_email_redaction() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "My email is user@example.com".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let rules = vec![
        RedactionRuleV1 {
            name: "email".to_string(),
            pattern: "email".to_string(),
            replacement: "[EMAIL]".to_string(),
        },
    ];

    apply_redactions(&mut prompt, &rules);
    
    assert!(prompt.messages[0].content.contains("[EMAIL]"));
    assert!(!prompt.messages[0].content.contains("@"));
}

#[test]
fn test_phone_redaction() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "Call me at +1-555-123-4567".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let rules = vec![
        RedactionRuleV1 {
            name: "phone".to_string(),
            pattern: "phone".to_string(),
            replacement: "[PHONE]".to_string(),
        },
    ];

    apply_redactions(&mut prompt, &rules);
    
    assert!(prompt.messages[0].content.contains("[PHONE]"));
    assert!(!prompt.messages[0].content.contains("+1-"));
}

#[test]
fn test_api_key_redaction() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "My API key is sk-1234567890abcdef".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let rules = vec![
        RedactionRuleV1 {
            name: "key".to_string(),
            pattern: "key".to_string(),
            replacement: "[KEY]".to_string(),
        },
    ];

    apply_redactions(&mut prompt, &rules);
    
    assert!(prompt.messages[0].content.contains("[KEY]"));
    assert!(!prompt.messages[0].content.contains("sk-"));
}

#[test]
fn test_multiple_redactions() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "Email: user@example.com, Phone: +1-555-123-4567, Key: sk-abcdef".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let rules = vec![
        RedactionRuleV1 {
            name: "email".to_string(),
            pattern: "email".to_string(),
            replacement: "[EMAIL]".to_string(),
        },
        RedactionRuleV1 {
            name: "phone".to_string(),
            pattern: "phone".to_string(),
            replacement: "[PHONE]".to_string(),
        },
        RedactionRuleV1 {
            name: "key".to_string(),
            pattern: "key".to_string(),
            replacement: "[KEY]".to_string(),
        },
    ];

    apply_redactions(&mut prompt, &rules);
    
    let content = &prompt.messages[0].content;
    assert!(content.contains("[EMAIL]"));
    assert!(content.contains("[PHONE]"));
    assert!(content.contains("[KEY]"));
    assert!(!content.contains("@"));
    assert!(!content.contains("+1-"));
    assert!(!content.contains("sk-"));
}

#[test]
fn test_redaction_idempotence() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "My email is user@example.com".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let rules = vec![
        RedactionRuleV1 {
            name: "email".to_string(),
            pattern: "email".to_string(),
            replacement: "[EMAIL]".to_string(),
        },
    ];

    // Apply redactions multiple times
    apply_redactions(&mut prompt, &rules);
    let content_after_first = prompt.messages[0].content.clone();
    
    apply_redactions(&mut prompt, &rules);
    let content_after_second = prompt.messages[0].content.clone();
    
    // Content should be the same after multiple applications
    assert_eq!(content_after_first, content_after_second);
    assert!(content_after_second.contains("[EMAIL]"));
}

#[test]
fn test_no_redaction_rules() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "My email is user@example.com".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let original_content = prompt.messages[0].content.clone();
    let rules = vec![];

    apply_redactions(&mut prompt, &rules);
    
    // Content should be unchanged
    assert_eq!(prompt.messages[0].content, original_content);
}

#[test]
fn test_redaction_with_multiple_messages() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "Email: user@example.com".to_string() 
            },
            MessageV1 { 
                role: ROLE_USER, 
                content: "Phone: +1-555-123-4567".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let rules = vec![
        RedactionRuleV1 {
            name: "email".to_string(),
            pattern: "email".to_string(),
            replacement: "[EMAIL]".to_string(),
        },
        RedactionRuleV1 {
            name: "phone".to_string(),
            pattern: "phone".to_string(),
            replacement: "[PHONE]".to_string(),
        },
    ];

    apply_redactions(&mut prompt, &rules);
    
    // First message should have email redacted
    assert!(prompt.messages[0].content.contains("[EMAIL]"));
    assert!(!prompt.messages[0].content.contains("@"));
    
    // Second message should have phone redacted
    assert!(prompt.messages[1].content.contains("[PHONE]"));
    assert!(!prompt.messages[1].content.contains("+1-"));
}

#[test]
fn test_redaction_pattern_matching() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "Contact: user@example.com, +1-555-123-4567".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let rules = vec![
        RedactionRuleV1 {
            name: "contact".to_string(),
            pattern: "contact".to_string(),
            replacement: "[CONTACT]".to_string(),
        },
    ];

    apply_redactions(&mut prompt, &rules);
    
    // Should match "contact" pattern
    assert!(prompt.messages[0].content.contains("[CONTACT]"));
    assert!(!prompt.messages[0].content.contains("Contact:"));
}

#[test]
fn test_redaction_case_sensitivity() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "EMAIL: user@example.com, email: admin@test.com".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let rules = vec![
        RedactionRuleV1 {
            name: "email".to_string(),
            pattern: "email".to_string(),
            replacement: "[EMAIL]".to_string(),
        },
    ];

    apply_redactions(&mut prompt, &rules);
    
    // Should match both "EMAIL" and "email"
    let content = &prompt.messages[0].content;
    assert!(content.contains("[EMAIL]"));
    assert!(!content.contains("@"));
}

#[test]
fn test_redaction_with_special_characters() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "Email: user.name+tag@example-domain.com".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let rules = vec![
        RedactionRuleV1 {
            name: "email".to_string(),
            pattern: "email".to_string(),
            replacement: "[EMAIL]".to_string(),
        },
    ];

    apply_redactions(&mut prompt, &rules);
    
    // Should handle complex email addresses
    assert!(prompt.messages[0].content.contains("[EMAIL]"));
    assert!(!prompt.messages[0].content.contains("@"));
}

#[test]
fn test_redaction_rule_ordering() {
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { 
                role: ROLE_USER, 
                content: "Email: user@example.com".to_string() 
            },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };

    let rules = vec![
        RedactionRuleV1 {
            name: "first".to_string(),
            pattern: "email".to_string(),
            replacement: "[FIRST]".to_string(),
        },
        RedactionRuleV1 {
            name: "second".to_string(),
            pattern: "email".to_string(),
            replacement: "[SECOND]".to_string(),
        },
    ];

    apply_redactions(&mut prompt, &rules);
    
    // Should apply rules in order (first rule wins)
    assert!(prompt.messages[0].content.contains("[FIRST]"));
    assert!(!prompt.messages[0].content.contains("[SECOND]"));
    assert!(!prompt.messages[0].content.contains("@"));
}
