use crate::secman::cap_flags::*;

/// Topic descriptor with static capability requirements
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicDesc {
    pub name: &'static str,
    pub id: u32,
    pub caps_required_pub: u64,
    pub caps_required_sub: u64,
    pub description: &'static str,
}

/// Static topic table generated at build time
pub static TOPICS: &[TopicDesc] = &[
    // Intent-related topics
    TopicDesc {
        name: "intent.created",
        id: 1,
        caps_required_pub: CAP_INTENT_SUBMIT,
        caps_required_sub: CAP_INTENT_QUERY,
        description: "New intent submitted to the system",
    },
    TopicDesc {
        name: "intent.previewed",
        id: 2,
        caps_required_pub: CAP_INTENT_SUBMIT,
        caps_required_sub: CAP_INTENT_QUERY,
        description: "Intent preview completed",
    },
    TopicDesc {
        name: "intent.completed",
        id: 3,
        caps_required_pub: CAP_INTENT_SUBMIT,
        caps_required_sub: CAP_INTENT_QUERY,
        description: "Intent execution completed",
    },
    TopicDesc {
        name: "intent.cancelled",
        id: 4,
        caps_required_pub: CAP_INTENT_SUBMIT,
        caps_required_sub: CAP_INTENT_QUERY,
        description: "Intent was cancelled",
    },
    
    // World Model topics
    TopicDesc {
        name: "wm.put",
        id: 10,
        caps_required_pub: CAP_WM_WRITE,
        caps_required_sub: CAP_WM_READ,
        description: "Facts added to world model",
    },
    TopicDesc {
        name: "wm.snapshot.new",
        id: 11,
        caps_required_pub: CAP_WM_WRITE,
        caps_required_sub: CAP_WM_READ,
        description: "New world model snapshot created",
    },
    TopicDesc {
        name: "wm.query",
        id: 12,
        caps_required_pub: CAP_WM_READ,
        caps_required_sub: CAP_WM_READ,
        description: "World model query executed",
    },
    
    // Skill-related topics
    TopicDesc {
        name: "skill.loaded",
        id: 20,
        caps_required_pub: CAP_SKILL_LOAD,
        caps_required_sub: CAP_SKILL_QUERY,
        description: "Skill loaded into runtime",
    },
    TopicDesc {
        name: "skill.invoked",
        id: 21,
        caps_required_pub: CAP_SKILL_INVOKE,
        caps_required_sub: CAP_SKILL_QUERY,
        description: "Skill invoked for preview",
    },
    TopicDesc {
        name: "skill.preview",
        id: 22,
        caps_required_pub: CAP_SKILL_INVOKE,
        caps_required_sub: CAP_SKILL_QUERY,
        description: "Skill preview result available",
    },
    TopicDesc {
        name: "skill.unloaded",
        id: 23,
        caps_required_pub: CAP_SKILL_LOAD,
        caps_required_sub: CAP_SKILL_QUERY,
        description: "Skill unloaded from runtime",
    },
    
    // System topics
    TopicDesc {
        name: "sys.timer",
        id: 30,
        caps_required_pub: 0, // System internal
        caps_required_sub: 0,  // System internal
        description: "Timer tick events",
    },
    TopicDesc {
        name: "sys.audit",
        id: 31,
        caps_required_pub: 0, // System internal
        caps_required_sub: 0,  // System internal
        description: "Audit event notifications",
    },
    
    // Device topics (future)
    TopicDesc {
        name: "device.connected",
        id: 40,
        caps_required_pub: 0, // TBD
        caps_required_sub: 0,  // TBD
        description: "Device connected to system",
    },
    TopicDesc {
        name: "device.disconnected",
        id: 41,
        caps_required_pub: 0, // TBD
        caps_required_sub: 0,  // TBD
        description: "Device disconnected from system",
    },
];

/// Find topic by name
pub fn find_topic(name: &str) -> Option<&'static TopicDesc> {
    TOPICS.iter().find(|t| t.name == name)
}

/// Find topic by ID
pub fn find_topic_by_id(id: u32) -> Option<&'static TopicDesc> {
    TOPICS.iter().find(|t| t.id == id)
}

/// Check if a topic name matches a pattern (for prefix subscriptions)
pub fn topic_matches_pattern(topic_name: &str, pattern: &str) -> bool {
    if pattern.ends_with(".*") {
        let prefix = &pattern[..pattern.len() - 2];
        topic_name.starts_with(prefix) && topic_name != prefix
    } else {
        topic_name == pattern
    }
}

/// Get all topics that match a pattern
pub fn get_matching_topics(pattern: &str) -> Vec<&'static TopicDesc> {
    TOPICS.iter()
        .filter(|t| topic_matches_pattern(t.name, pattern))
        .collect()
}

/// Validate topic name format
pub fn is_valid_topic_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    
    // Must start with alphanumeric or lowercase letter
    if !name.chars().next().unwrap().is_ascii_lowercase() {
        return false;
    }
    
    // Can contain: lowercase letters, numbers, dots, underscores
    name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_')
}

/// Get topic schema hash (computed at build time)
pub fn get_topics_schema_hash() -> [u8; 32] {
    // This would be computed at build time by the generator
    // For now, return a placeholder hash
    let mut hash = [0u8; 32];
    hash[0] = b'E';
    hash[1] = b'V';
    hash[2] = b'T';
    hash[3] = b'0';
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_topic() {
        let topic = find_topic("intent.created");
        assert!(topic.is_some());
        assert_eq!(topic.unwrap().id, 1);
    }

    #[test]
    fn test_find_topic_by_id() {
        let topic = find_topic_by_id(10);
        assert!(topic.is_some());
        assert_eq!(topic.unwrap().name, "wm.put");
    }

    #[test]
    fn test_topic_matches_pattern() {
        assert!(topic_matches_pattern("intent.created", "intent.*"));
        assert!(topic_matches_pattern("wm.put", "wm.*"));
        assert!(!topic_matches_pattern("intent.created", "wm.*"));
        assert!(topic_matches_pattern("skill.loaded", "skill.*"));
    }

    #[test]
    fn test_get_matching_topics() {
        let intent_topics = get_matching_topics("intent.*");
        assert_eq!(intent_topics.len(), 4);
        assert!(intent_topics.iter().all(|t| t.name.starts_with("intent.")));
    }

    #[test]
    fn test_is_valid_topic_name() {
        assert!(is_valid_topic_name("intent.created"));
        assert!(is_valid_topic_name("wm.put"));
        assert!(is_valid_topic_name("skill.loaded"));
        assert!(!is_valid_topic_name("Intent.created")); // Capital letter
        assert!(!is_valid_topic_name("")); // Empty
        assert!(!is_valid_topic_name("a".repeat(65))); // Too long
    }
}
