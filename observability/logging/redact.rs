use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Redaction configuration for PII protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionConfig {
    /// Enable redaction
    pub enabled: bool,
    /// Redaction mode
    pub mode: RedactionMode,
    /// Custom redaction rules
    pub custom_rules: Vec<RedactionRule>,
    /// Sensitive field patterns
    pub sensitive_patterns: Vec<String>,
    /// Redaction placeholder
    pub placeholder: String,
    /// Enable regex-based redaction
    pub enable_regex: bool,
    /// Case-sensitive matching
    pub case_sensitive: bool,
    /// Enable partial redaction (e.g., show only last 4 digits)
    pub enable_partial: bool,
    /// Partial redaction length
    pub partial_length: usize,
}

/// Redaction modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedactionMode {
    /// No redaction
    None,
    /// Replace with placeholder
    Placeholder,
    /// Hash the value
    Hash,
    /// Drop the field entirely
    Drop,
    /// Partial redaction (show only part of the value)
    Partial,
}

/// Redaction rule for specific fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRule {
    /// Field name pattern to match
    pub field_pattern: String,
    /// Redaction mode for this rule
    pub mode: RedactionMode,
    /// Custom placeholder for this rule
    pub custom_placeholder: Option<String>,
    /// Enable regex matching for field names
    pub use_regex: bool,
    /// Priority (higher = more specific)
    pub priority: u8,
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: RedactionMode::Placeholder,
            custom_rules: vec![],
            sensitive_patterns: vec![
                // Common PII patterns
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "key".to_string(),
                "credential".to_string(),
                "auth".to_string(),
                "private".to_string(),
                "sensitive".to_string(),
                
                // Personal information
                "email".to_string(),
                "phone".to_string(),
                "address".to_string(),
                "ssn".to_string(),
                "social_security".to_string(),
                "credit_card".to_string(),
                "card_number".to_string(),
                "cvv".to_string(),
                
                // User identifiers
                "user_id".to_string(),
                "user_name".to_string(),
                "username".to_string(),
                "login".to_string(),
                "account".to_string(),
                
                // Financial information
                "amount".to_string(),
                "balance".to_string(),
                "account_number".to_string(),
                "routing".to_string(),
                "swift".to_string(),
                "iban".to_string(),
                
                // API and security
                "api_key".to_string(),
                "access_token".to_string(),
                "refresh_token".to_string(),
                "session_id".to_string(),
                "cookie".to_string(),
                "authorization".to_string(),
                
                // Network and location
                "ip_address".to_string(),
                "ip".to_string(),
                "mac_address".to_string(),
                "gps".to_string(),
                "latitude".to_string(),
                "longitude".to_string(),
                "location".to_string(),
                
                // Business sensitive
                "contract".to_string(),
                "agreement".to_string(),
                "confidential".to_string(),
                "proprietary".to_string(),
                "trade_secret".to_string(),
            ],
            placeholder: "[REDACTED]".to_string(),
            enable_regex: true,
            case_sensitive: false,
            enable_partial: false,
            partial_length: 4,
        }
    }
}

impl Default for RedactionRule {
    fn default() -> Self {
        Self {
            field_pattern: "".to_string(),
            mode: RedactionMode::Placeholder,
            custom_placeholder: None,
            use_regex: false,
            priority: 0,
        }
    }
}

/// Redaction engine for processing log fields
#[derive(Debug)]
pub struct RedactionEngine {
    config: RedactionConfig,
    compiled_patterns: Vec<Regex>,
    custom_rules: Vec<RedactionRule>,
}

impl RedactionEngine {
    /// Create a new redaction engine
    pub fn new(config: RedactionConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut engine = Self {
            config: config.clone(),
            compiled_patterns: Vec::new(),
            custom_rules: config.custom_rules.clone(),
        };

        // Compile regex patterns if enabled
        if config.enable_regex {
            for pattern in &config.sensitive_patterns {
                let regex = if config.case_sensitive {
                    Regex::new(pattern)?
                } else {
                    Regex::new(&format!("(?i){}", pattern))?
                };
                engine.compiled_patterns.push(regex);
            }
        }

        // Sort custom rules by priority
        engine.custom_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(engine)
    }

    /// Redact sensitive fields in a HashMap
    pub fn redact_fields(&self, fields: &HashMap<String, Value>) -> HashMap<String, Value> {
        if !self.config.enabled {
            return fields.clone();
        }

        let mut redacted_fields = HashMap::new();

        for (key, value) in fields {
            let redacted_value = self.redact_field(key, value);
            redacted_fields.insert(key.clone(), redacted_value);
        }

        redacted_fields
    }

    /// Redact a single field
    pub fn redact_field(&self, field_name: &str, value: &Value) -> Value {
        // Check custom rules first (higher priority)
        if let Some(rule) = self.find_matching_custom_rule(field_name) {
            return self.apply_redaction_rule(value, rule);
        }

        // Check if field matches sensitive patterns
        if self.is_sensitive_field(field_name) {
            return self.apply_redaction_mode(value, self.config.mode);
        }

        // Field is not sensitive, return as-is
        value.clone()
    }

    /// Check if a field name matches sensitive patterns
    fn is_sensitive_field(&self, field_name: &str) -> bool {
        let field_name_lower = if self.config.case_sensitive {
            field_name.to_string()
        } else {
            field_name.to_lowercase()
        };

        // Check compiled regex patterns
        for pattern in &self.compiled_patterns {
            if pattern.is_match(&field_name_lower) {
                return true;
            }
        }

        // Check simple string patterns
        for pattern in &self.config.sensitive_patterns {
            if self.config.case_sensitive {
                if field_name_lower.contains(pattern) {
                    return true;
                }
            } else {
                if field_name_lower.contains(&pattern.to_lowercase()) {
                    return true;
                }
            }
        }

        false
    }

    /// Find matching custom rule for a field
    fn find_matching_custom_rule(&self, field_name: &str) -> Option<&RedactionRule> {
        for rule in &self.custom_rules {
            if rule.use_regex {
                if let Ok(regex) = Regex::new(&rule.field_pattern) {
                    if regex.is_match(field_name) {
                        return Some(rule);
                    }
                }
            } else {
                if field_name.contains(&rule.field_pattern) {
                    return Some(rule);
                }
            }
        }
        None
    }

    /// Apply redaction rule to a value
    fn apply_redaction_rule(&self, value: &Value, rule: &RedactionRule) -> Value {
        let mode = rule.mode;
        let placeholder = rule.custom_placeholder.as_ref().unwrap_or(&self.config.placeholder);
        
        match mode {
            RedactionMode::None => value.clone(),
            RedactionMode::Placeholder => Value::String(placeholder.clone()),
            RedactionMode::Hash => self.hash_value(value),
            RedactionMode::Drop => Value::Null,
            RedactionMode::Partial => self.partial_redact(value),
        }
    }

    /// Apply redaction mode to a value
    fn apply_redaction_mode(&self, value: &Value, mode: RedactionMode) -> Value {
        match mode {
            RedactionMode::None => value.clone(),
            RedactionMode::Placeholder => Value::String(self.config.placeholder.clone()),
            RedactionMode::Hash => self.hash_value(value),
            RedactionMode::Drop => Value::Null,
            RedactionMode::Partial => self.partial_redact(value),
        }
    }

    /// Hash a value for redaction
    fn hash_value(&self, value: &Value) -> Value {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        value.to_string().hash(&mut hasher);
        let hash = hasher.finish();
        
        Value::String(format!("hash_{:x}", hash))
    }

    /// Partially redact a value (show only part of it)
    fn partial_redact(&self, value: &Value) -> Value {
        if !self.config.enable_partial {
            return Value::String(self.config.placeholder.clone());
        }

        let value_str = value.to_string();
        if value_str.len() <= self.config.partial_length {
            return Value::String("*".repeat(value_str.len()));
        }

        let visible_part = &value_str[value_str.len() - self.config.partial_length..];
        Value::String(format!("{}{}", "*".repeat(value_str.len() - self.config.partial_length), visible_part))
    }

    /// Add a custom redaction rule
    pub fn add_custom_rule(&mut self, rule: RedactionRule) {
        self.custom_rules.push(rule);
        // Re-sort by priority
        self.custom_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove a custom redaction rule
    pub fn remove_custom_rule(&mut self, field_pattern: &str) {
        self.custom_rules.retain(|rule| rule.field_pattern != field_pattern);
    }

    /// Test if a field would be redacted
    pub fn would_redact(&self, field_name: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Check custom rules first
        if self.find_matching_custom_rule(field_name).is_some() {
            return true;
        }

        // Check sensitive patterns
        self.is_sensitive_field(field_name)
    }

    /// Get redaction statistics
    pub fn get_stats(&self) -> RedactionStats {
        RedactionStats {
            total_patterns: self.config.sensitive_patterns.len(),
            compiled_patterns: self.compiled_patterns.len(),
            custom_rules: self.custom_rules.len(),
            enabled: self.config.enabled,
            mode: self.config.mode,
        }
    }
}

/// Redaction statistics
#[derive(Debug, Clone, Serialize)]
pub struct RedactionStats {
    pub total_patterns: usize,
    pub compiled_patterns: usize,
    pub custom_rules: usize,
    pub enabled: bool,
    pub mode: RedactionMode,
}

/// Builder for creating redaction rules
#[derive(Debug)]
pub struct RedactionRuleBuilder {
    rule: RedactionRule,
}

impl RedactionRuleBuilder {
    /// Create a new rule builder
    pub fn new(field_pattern: String) -> Self {
        Self {
            rule: RedactionRule {
                field_pattern,
                ..Default::default()
            },
        }
    }

    /// Set redaction mode
    pub fn mode(mut self, mode: RedactionMode) -> Self {
        self.rule.mode = mode;
        self
    }

    /// Set custom placeholder
    pub fn placeholder(mut self, placeholder: String) -> Self {
        self.rule.custom_placeholder = Some(placeholder);
        self
    }

    /// Enable regex matching
    pub fn regex(mut self) -> Self {
        self.rule.use_regex = true;
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: u8) -> Self {
        self.rule.priority = priority;
        self
    }

    /// Build the rule
    pub fn build(self) -> RedactionRule {
        self.rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_redaction_config_default() {
        let config = RedactionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.mode, RedactionMode::Placeholder);
        assert!(!config.sensitive_patterns.is_empty());
        assert_eq!(config.placeholder, "[REDACTED]");
    }

    #[test]
    fn test_redaction_engine_creation() {
        let config = RedactionConfig::default();
        let engine = RedactionEngine::new(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_sensitive_field_detection() {
        let config = RedactionConfig::default();
        let engine = RedactionEngine::new(config).unwrap();
        
        assert!(engine.would_redact("password"));
        assert!(engine.would_redact("user_password"));
        assert!(engine.would_redact("PASSWORD"));
        assert!(engine.would_redact("api_key"));
        assert!(!engine.would_redact("username"));
        assert!(!engine.would_redact("normal_field"));
    }

    #[test]
    fn test_field_redaction() {
        let config = RedactionConfig::default();
        let engine = RedactionEngine::new(config).unwrap();
        
        let mut fields = HashMap::new();
        fields.insert("username".to_string(), Value::String("john_doe".to_string()));
        fields.insert("password".to_string(), Value::String("secret123".to_string()));
        fields.insert("email".to_string(), Value::String("john@example.com".to_string()));
        
        let redacted = engine.redact_fields(&fields);
        
        assert_eq!(redacted["username"], Value::String("john_doe".to_string()));
        assert_eq!(redacted["password"], Value::String("[REDACTED]".to_string()));
        assert_eq!(redacted["email"], Value::String("[REDACTED]".to_string()));
    }

    #[test]
    fn test_custom_redaction_rules() {
        let mut config = RedactionConfig::default();
        config.custom_rules.push(
            RedactionRuleBuilder::new("custom_field".to_string())
                .mode(RedactionMode::Hash)
                .priority(10)
                .build()
        );
        
        let engine = RedactionEngine::new(config).unwrap();
        
        assert!(engine.would_redact("custom_field"));
        assert!(engine.would_redact("my_custom_field"));
        
        let mut fields = HashMap::new();
        fields.insert("custom_field".to_string(), Value::String("sensitive_value".to_string()));
        
        let redacted = engine.redact_fields(&fields);
        assert!(redacted["custom_field"].as_str().unwrap().starts_with("hash_"));
    }

    #[test]
    fn test_partial_redaction() {
        let mut config = RedactionConfig::default();
        config.enable_partial = true;
        config.partial_length = 4;
        
        let engine = RedactionEngine::new(config).unwrap();
        
        let value = Value::String("1234567890".to_string());
        let redacted = engine.redact_field("credit_card", &value);
        
        assert_eq!(redacted, Value::String("******7890".to_string()));
    }

    #[test]
    fn test_hash_redaction() {
        let mut config = RedactionConfig::default();
        config.mode = RedactionMode::Hash;
        
        let engine = RedactionEngine::new(config).unwrap();
        
        let value = Value::String("secret_value".to_string());
        let redacted = engine.redact_field("password", &value);
        
        assert!(redacted.as_str().unwrap().starts_with("hash_"));
    }

    #[test]
    fn test_drop_redaction() {
        let mut config = RedactionConfig::default();
        config.mode = RedactionMode::Drop;
        
        let engine = RedactionEngine::new(config).unwrap();
        
        let value = Value::String("secret_value".to_string());
        let redacted = engine.redact_field("password", &value);
        
        assert_eq!(redacted, Value::Null);
    }

    #[test]
    fn test_redaction_stats() {
        let config = RedactionConfig::default();
        let engine = RedactionEngine::new(config).unwrap();
        
        let stats = engine.get_stats();
        assert!(stats.enabled);
        assert_eq!(stats.mode, RedactionMode::Placeholder);
        assert!(stats.total_patterns > 0);
        assert!(stats.compiled_patterns > 0);
    }
}
