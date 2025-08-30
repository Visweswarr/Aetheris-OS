# Unit Tests for Wallet Spend Limits Policy
#
# Tests various scenarios for wallet spending controls including:
# - Daily limit enforcement
# - Transaction size limits
# - Balance sufficiency
# - Risk assessment
# - Velocity limiting

package polymera.wallet.spend_limits

import rego.v1

# Test data setup
test_users := {
    "user123": {
        "tier": "premium",
        "spending_history": {
            "average_transaction": {
                "USD": 500.0,
                "BTC": 0.1
            }
        },
        "activity_patterns": {
            "typical_hours": [8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18]
        },
        "recipients": ["merchant456", "friend789"],
        "typical_locations": [
            {"latitude": 40.7128, "longitude": -74.0060}  # NYC
        ],
        "trusted_devices": ["device_abc123"]
    },
    "user456": {
        "tier": "basic",
        "spending_history": {
            "average_transaction": {
                "USD": 50.0
            }
        },
        "activity_patterns": {
            "typical_hours": [9, 10, 11, 12, 13, 14, 15, 16, 17]
        },
        "recipients": [],
        "typical_locations": [
            {"latitude": 34.0522, "longitude": -118.2437}  # LA
        ],
        "trusted_devices": ["device_def456"]
    }
}

test_policy_config := {
    "tiers": {
        "premium": {
            "daily_limits": {
                "USD": 10000.0,
                "BTC": 1.0
            },
            "max_transaction": {
                "USD": 5000.0,
                "BTC": 0.5
            },
            "velocity_limits": {
                "transactions_per_5min": 10
            },
            "max_risk_score": 50
        },
        "basic": {
            "daily_limits": {
                "USD": 1000.0,
                "BTC": 0.1
            },
            "max_transaction": {
                "USD": 500.0,
                "BTC": 0.05
            },
            "velocity_limits": {
                "transactions_per_5min": 3
            },
            "max_risk_score": 30
        },
        "default": {
            "daily_limits": {
                "USD": 100.0
            },
            "max_transaction": {
                "USD": 50.0
            },
            "velocity_limits": {
                "transactions_per_5min": 1
            },
            "max_risk_score": 20
        }
    },
    "fees": {
        "USD": {"base": 1.0, "percentage": 0.5},
        "BTC": {"base": 0.001, "percentage": 0.1}
    }
}

test_balances := {
    "user123": {
        "USD": 15000.0,
        "BTC": 2.0
    },
    "user456": {
        "USD": 800.0,
        "BTC": 0.05
    }
}

test_transactions := {
    "user123": {
        "2024-01-15": {
            "USD": [
                {"amount": 1000.0, "timestamp": "2024-01-15T10:00:00Z"},
                {"amount": 500.0, "timestamp": "2024-01-15T11:00:00Z"}
            ]
        }
    },
    "user456": {
        "2024-01-15": {
            "USD": []
        }
    }
}

# Mock data for testing
data := {
    "users": test_users,
    "policy_config": test_policy_config,
    "balances": test_balances,
    "transactions": test_transactions,
    "security": {
        "blacklisted_users": ["blacklisted_user"]
    }
}

# Test cases for ALLOW scenarios

test_allow_valid_spend_within_limits if {
    allow with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 1000.0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345",
        "recipient_id": "merchant456",
        "location": {"latitude": 40.7128, "longitude": -74.0060},
        "device_id": "device_abc123"
    }
    with data as data
}

test_allow_balance_inquiry if {
    allow with input as {
        "operation": "balance_inquiry",
        "user_id": "user123",
        "amount": 0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345"
    }
    with data as data
}

test_allow_system_operation if {
    allow with input as {
        "operation": "audit",
        "user_id": "system",
        "amount": 0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "system_role": "internal",
        "system_token": "sys_audit_token_12345"
    }
    with data as data
}

test_allow_small_transaction_basic_user if {
    allow with input as {
        "operation": "spend",
        "user_id": "user456",
        "amount": 100.0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_67890",
        "recipient_id": "new_merchant",
        "location": {"latitude": 34.0522, "longitude": -118.2437},
        "device_id": "device_def456"
    }
    with data as data
}

test_allow_crypto_transaction if {
    allow with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 0.1,
        "currency": "BTC",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345",
        "recipient_id": "merchant456",
        "location": {"latitude": 40.7128, "longitude": -74.0060},
        "device_id": "device_abc123"
    }
    with data as data
}

# Test cases for DENY scenarios

test_deny_invalid_input_missing_user if {
    not allow with input as {
        "operation": "spend",
        "amount": 100.0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_deny_invalid_currency if {
    not allow with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 100.0,
        "currency": "INVALID",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345"
    }
    with data as data
}

test_deny_negative_amount if {
    not allow with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": -100.0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345"
    }
    with data as data
}

test_deny_daily_limit_exceeded if {
    not allow with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 9000.0,  # This plus existing 1500 exceeds 10000 limit
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345",
        "recipient_id": "merchant456",
        "location": {"latitude": 40.7128, "longitude": -74.0060},
        "device_id": "device_abc123"
    }
    with data as data
}

test_deny_transaction_limit_exceeded if {
    not allow with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 6000.0,  # Exceeds 5000 max transaction limit
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345",
        "recipient_id": "merchant456",
        "location": {"latitude": 40.7128, "longitude": -74.0060},
        "device_id": "device_abc123"
    }
    with data as data
}

test_deny_insufficient_balance if {
    not allow with input as {
        "operation": "spend",
        "user_id": "user456",
        "amount": 900.0,  # User456 only has 800 USD
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_67890",
        "recipient_id": "merchant456",
        "location": {"latitude": 34.0522, "longitude": -118.2437},
        "device_id": "device_def456"
    }
    with data as data
}

test_deny_blacklisted_user if {
    not allow with input as {
        "operation": "spend",
        "user_id": "blacklisted_user",
        "amount": 100.0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345"
    }
    with data as data
}

test_deny_basic_user_exceeds_limit if {
    not allow with input as {
        "operation": "spend",
        "user_id": "user456",
        "amount": 600.0,  # Exceeds basic user's 500 limit
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_67890",
        "recipient_id": "merchant456",
        "location": {"latitude": 34.0522, "longitude": -118.2437},
        "device_id": "device_def456"
    }
    with data as data
}

# Test rationale generation

test_rationale_for_approval if {
    reasons := rationale with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 1000.0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345",
        "recipient_id": "merchant456",
        "location": {"latitude": 40.7128, "longitude": -74.0060},
        "device_id": "device_abc123"
    }
    with data as data
    
    count(reasons) > 0
    "transaction approved" in reasons
}

test_rationale_for_denial_daily_limit if {
    reasons := rationale with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 9000.0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345",
        "recipient_id": "merchant456",
        "location": {"latitude": 40.7128, "longitude": -74.0060},
        "device_id": "device_abc123"
    }
    with data as data
    
    count(reasons) > 0
    some reason in reasons
    contains(reason, "daily limit exceeded")
}

test_rationale_for_denial_insufficient_balance if {
    reasons := rationale with input as {
        "operation": "spend",
        "user_id": "user456",
        "amount": 900.0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_67890",
        "recipient_id": "merchant456",
        "location": {"latitude": 34.0522, "longitude": -118.2437},
        "device_id": "device_def456"
    }
    with data as data
    
    count(reasons) > 0
    some reason in reasons
    contains(reason, "insufficient balance")
}

# Test risk assessment functions

test_risk_calculation_normal_transaction if {
    risk_factors := calculate_risk_factors with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 500.0,  # Normal amount
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "recipient_id": "merchant456",  # Known recipient
        "location": {"latitude": 40.7128, "longitude": -74.0060},  # Typical location
        "device_id": "device_abc123"  # Trusted device
    }
    with data as data
    
    total_risk := sum([factor.score | factor := risk_factors[_]])
    total_risk < 20  # Should be low risk
}

test_risk_calculation_high_risk_transaction if {
    risk_factors := calculate_risk_factors with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 5000.0,  # 10x normal amount
        "currency": "USD",
        "timestamp": "2024-01-15T02:00:00Z",  # Unusual time (2 AM)
        "recipient_id": "unknown_recipient",  # New recipient
        "location": {"latitude": 51.5074, "longitude": -0.1278},  # Different location (London)
        "device_id": "unknown_device"  # New device
    }
    with data as data
    
    total_risk := sum([factor.score | factor := risk_factors[_]])
    total_risk > 50  # Should be high risk
}

# Test helper functions

test_calculate_daily_spend if {
    daily_spend := calculate_daily_spend("user123", "USD") with data as data
    daily_spend == 1500.0  # 1000 + 500 from test data
}

test_calculate_daily_spend_no_transactions if {
    daily_spend := calculate_daily_spend("user456", "USD") with data as data
    daily_spend == 0  # No transactions
}

test_get_user_limits_premium if {
    limits := get_user_limits("user123") with data as data
    limits.daily_limits.USD == 10000.0
    limits.max_transaction.USD == 5000.0
}

test_get_user_limits_basic if {
    limits := get_user_limits("user456") with data as data
    limits.daily_limits.USD == 1000.0
    limits.max_transaction.USD == 500.0
}

test_get_user_balance if {
    balance := get_user_balance("user123", "USD") with data as data
    balance == 15000.0
}

test_calculate_fees if {
    fee := calculate_fees(1000.0, "USD") with data as data
    fee == 6.0  # 1.0 base + (1000 * 0.5 / 100) = 1.0 + 5.0
}

# Emergency override tests

test_allow_emergency_override if {
    allow with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 20000.0,  # Exceeds all limits
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345",
        "emergency_token": "emergency_12345",
        "admin_user_id": "admin123"
    }
    with data as {
        "users": test_users,
        "policy_config": test_policy_config,
        "balances": test_balances,
        "transactions": test_transactions,
        "admins": {
            "admin123": {"emergency_access": true}
        }
    }
}

test_deny_invalid_emergency_override if {
    not allow with input as {
        "operation": "spend",
        "user_id": "user123",
        "amount": 20000.0,
        "currency": "USD",
        "timestamp": "2024-01-15T14:00:00Z",
        "session_token": "valid_session_token_12345",
        "emergency_token": "invalid_token",
        "admin_user_id": "admin123"
    }
    with data as {
        "users": test_users,
        "policy_config": test_policy_config,
        "balances": test_balances,
        "transactions": test_transactions,
        "admins": {
            "admin123": {"emergency_access": false}
        }
    }
}
