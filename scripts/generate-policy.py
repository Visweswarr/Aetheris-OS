#!/usr/bin/env python3
"""
Script to generate AI Core Service capability policy file

This script generates a CBOR-encoded policy file that defines
all capabilities, resources, and access rules for the AI Core Service.
"""

import cbor2
import time
import os
from typing import Dict, List, Any

def create_ai_core_policy() -> Dict[str, Any]:
    """Create the AI Core Service capability policy"""
    
    # Define capabilities
    capabilities = {
        "ai:chat": {
            "name": "ai:chat",
            "description": "Chat with AI assistant",
            "category": "ai",
            "required_resources": ["ai_core"],
            "allowed_actions": ["generate_response"],
            "conditions": [],
            "metadata": {}
        },
        "ai:tools": {
            "name": "ai:tools",
            "description": "Execute AI tools",
            "category": "ai",
            "required_resources": ["tools"],
            "allowed_actions": ["execute"],
            "conditions": [],
            "metadata": {}
        },
        "ai:intents": {
            "name": "ai:intents",
            "description": "Execute system intents",
            "category": "ai",
            "required_resources": ["system"],
            "allowed_actions": ["execute"],
            "conditions": [],
            "metadata": {}
        },
        "ai:memory": {
            "name": "ai:memory",
            "description": "Access AI memory store",
            "category": "ai",
            "required_resources": ["memory"],
            "allowed_actions": ["read", "write", "delete"],
            "conditions": [],
            "metadata": {}
        },
        "ai:admin": {
            "name": "ai:admin",
            "description": "Administrative access to AI Core Service",
            "category": "admin",
            "required_resources": ["ai_core"],
            "allowed_actions": ["*"],
            "conditions": [],
            "metadata": {}
        }
    }
    
    # Define resources
    resources = {
        "ai_core": {
            "name": "ai_core",
            "description": "AI Core Service",
            "resource_type": "service",
            "pattern": "ai_core",
            "attributes": {},
            "metadata": {}
        },
        "tools": {
            "name": "tools",
            "description": "AI tools",
            "resource_type": "tool",
            "pattern": "tools:*",
            "attributes": {},
            "metadata": {}
        },
        "system": {
            "name": "system",
            "description": "System resources",
            "resource_type": "system",
            "pattern": "system:*",
            "attributes": {},
            "metadata": {}
        },
        "memory": {
            "name": "memory",
            "description": "AI memory store",
            "resource_type": "storage",
            "pattern": "memory:*",
            "attributes": {},
            "metadata": {}
        }
    }
    
    # Define access rules
    access_rules = [
        {
            "rule_id": "rule_001",
            "name": "Allow AI Chat",
            "description": "Allow users to chat with AI assistant",
            "priority": 100,
            "effect": "allow",
            "required_capabilities": ["ai:chat"],
            "required_resources": ["ai_core"],
            "required_actions": ["generate_response"],
            "conditions": [],
            "metadata": {}
        },
        {
            "rule_id": "rule_002",
            "name": "Allow Tool Execution",
            "description": "Allow users to execute AI tools",
            "priority": 100,
            "effect": "allow",
            "required_capabilities": ["ai:tools"],
            "required_resources": ["tools"],
            "required_actions": ["execute"],
            "conditions": [],
            "metadata": {}
        },
        {
            "rule_id": "rule_003",
            "name": "Allow System Intents",
            "description": "Allow users to execute system intents",
            "priority": 100,
            "effect": "allow",
            "required_capabilities": ["ai:intents"],
            "required_resources": ["system"],
            "required_actions": ["execute"],
            "conditions": [],
            "metadata": {}
        },
        {
            "rule_id": "rule_004",
            "name": "Allow Memory Access",
            "description": "Allow users to access AI memory store",
            "priority": 100,
            "effect": "allow",
            "required_capabilities": ["ai:memory"],
            "required_resources": ["memory"],
            "required_actions": ["read", "write", "delete"],
            "conditions": [],
            "metadata": {}
        },
        {
            "rule_id": "rule_005",
            "name": "Allow Admin Access",
            "description": "Allow administrators full access",
            "priority": 200,
            "effect": "allow",
            "required_capabilities": ["ai:admin"],
            "required_resources": ["ai_core"],
            "required_actions": ["*"],
            "conditions": [],
            "metadata": {}
        }
    ]
    
    # Define issuers
    issuers = {
        "aetheris-system": {
            "name": "Aetheris System",
            "description": "System issuer for AI Core Service",
            "trusted": True,
            "max_token_lifetime": 3600,  # 1 hour
            "allowed_capabilities": [
                "ai:chat",
                "ai:tools",
                "ai:intents",
                "ai:memory",
                "ai:admin"
            ],
            "metadata": {}
        }
    }
    
    # Create the complete policy
    policy = {
        "version": "1.0.0",
        "metadata": {
            "name": "AI Core Service Capability Policy",
            "description": "Capability policy for AI Core Service with deny-by-default security model",
            "created_at": int(time.time()),
            "updated_at": int(time.time()),
            "author": "Aetheris OS Team",
            "version": "1.0.0"
        },
        "default_policy": {
            "default_action": "deny",
            "default_reason": "Access denied by default policy",
            "log_denied": True,
            "audit_all": True
        },
        "capabilities": capabilities,
        "resources": resources,
        "access_rules": access_rules,
        "issuers": issuers
    }
    
    return policy

def main():
    """Generate the policy file"""
    # Create the policy
    policy = create_ai_core_policy()
    
    # Ensure the directory exists
    os.makedirs("configs/caps", exist_ok=True)
    
    # Serialize to CBOR
    cbor_data = cbor2.dumps(policy)
    
    # Write to file
    with open("configs/caps/ai_core.policy.cbor", "wb") as f:
        f.write(cbor_data)
    
    print("Generated AI Core Service capability policy file: configs/caps/ai_core.policy.cbor")
    print(f"Policy contains {len(policy['capabilities'])} capabilities, {len(policy['resources'])} resources, {len(policy['access_rules'])} access rules")
    print(f"File size: {len(cbor_data)} bytes")

if __name__ == "__main__":
    main()
