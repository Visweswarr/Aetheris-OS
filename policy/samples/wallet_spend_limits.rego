# Wallet Spend Limits Policy
# 
# Enforces spending limits for wallet operations including:
# - Daily spending limits per user
# - Transaction amount limits
# - Velocity-based rate limiting
# - Account balance requirements
# - Risk-based spending controls

package polymera.wallet.spend_limits

import rego.v1

# Default deny - all spend requests must be explicitly allowed
default allow := false

# Main authorization decision
allow if {
    input.operation == "spend"
    valid_input
    within_daily_limit
    within_transaction_limit
    sufficient_balance
    within_velocity_limits
    risk_score_acceptable
    not blacklisted_user
}

# Allow balance inquiries without spend restrictions
allow if {
    input.operation == "balance_inquiry"
    valid_input
    authenticated_user
}

# Allow internal system operations with proper authorization
allow if {
    input.operation in ["audit", "reconciliation"]
    input.system_role == "internal"
    valid_system_token
}

# Input validation
valid_input if {
    input.user_id
    input.amount >= 0
    input.currency in ["USD", "EUR", "BTC", "ETH", "POLY"]
    input.timestamp
    time.parse_rfc3339_ns(input.timestamp)
}

authenticated_user if {
    input.user_id
    input.session_token
    # In production, verify session token with keystore
    count(input.session_token) > 10
}

valid_system_token if {
    input.system_token
    # In production, verify system token cryptographically
    startswith(input.system_token, "sys_")
}

# Daily spending limit enforcement
within_daily_limit if {
    daily_spend := calculate_daily_spend(input.user_id, input.currency)
    user_limits := get_user_limits(input.user_id)
    currency_limit := user_limits.daily_limits[input.currency]
    
    (daily_spend + input.amount) <= currency_limit
}

# Transaction amount limits
within_transaction_limit if {
    user_limits := get_user_limits(input.user_id)
    max_transaction := user_limits.max_transaction[input.currency]
    
    input.amount <= max_transaction
}

# Balance sufficiency check
sufficient_balance if {
    current_balance := get_user_balance(input.user_id, input.currency)
    required_amount := input.amount + calculate_fees(input.amount, input.currency)
    
    current_balance >= required_amount
}

# Velocity-based rate limiting
within_velocity_limits if {
    recent_transactions := get_recent_transactions(input.user_id, 300) # 5 minutes
    transaction_count := count(recent_transactions)
    
    user_limits := get_user_limits(input.user_id)
    max_velocity := user_limits.velocity_limits.transactions_per_5min
    
    transaction_count < max_velocity
}

# Risk score assessment
risk_score_acceptable if {
    risk_factors := calculate_risk_factors
    total_risk_score := sum([factor.score | factor := risk_factors[_]])
    
    user_limits := get_user_limits(input.user_id)
    max_risk_score := user_limits.max_risk_score
    
    total_risk_score <= max_risk_score
}

# Blacklist check
blacklisted_user if {
    blacklist := data.security.blacklisted_users
    input.user_id in blacklist
}

not blacklisted_user if {
    not blacklisted_user
}

# Helper functions for calculations
calculate_daily_spend(user_id, currency) := total if {
    today := time.format(time.now_ns(), "2006-01-02")
    transactions := data.transactions[user_id][today][currency]
    total := sum([tx.amount | tx := transactions[_]])
}

calculate_daily_spend(user_id, currency) := 0 if {
    today := time.format(time.now_ns(), "2006-01-02")
    not data.transactions[user_id][today][currency]
}

get_user_limits(user_id) := limits if {
    user_tier := data.users[user_id].tier
    limits := data.policy_config.tiers[user_tier]
}

get_user_limits(user_id) := data.policy_config.tiers.default if {
    not data.users[user_id].tier
}

get_user_balance(user_id, currency) := balance if {
    balance := data.balances[user_id][currency]
}

get_user_balance(user_id, currency) := 0 if {
    not data.balances[user_id][currency]
}

get_recent_transactions(user_id, seconds_ago) := transactions if {
    cutoff_time := time.now_ns() - (seconds_ago * 1000000000)
    all_transactions := data.transactions[user_id]
    
    transactions := [tx |
        tx := all_transactions[_][_][_][_]
        time.parse_rfc3339_ns(tx.timestamp) > cutoff_time
    ]
}

calculate_fees(amount, currency) := fee if {
    fee_config := data.policy_config.fees[currency]
    base_fee := fee_config.base
    percentage_fee := amount * fee_config.percentage / 100
    fee := base_fee + percentage_fee
}

calculate_risk_factors := factors if {
    factors := [
        {"name": "unusual_amount", "score": unusual_amount_risk},
        {"name": "unusual_time", "score": unusual_time_risk},
        {"name": "new_recipient", "score": new_recipient_risk},
        {"name": "geographic", "score": geographic_risk},
        {"name": "device", "score": device_risk}
    ]
}

unusual_amount_risk := score if {
    user_history := data.users[input.user_id].spending_history
    avg_amount := user_history.average_transaction[input.currency]
    
    ratio := input.amount / avg_amount
    score := case ratio {
        ratio > 10 := 50
        ratio > 5 := 30
        ratio > 2 := 10
        default := 0
    }
}

unusual_time_risk := score if {
    current_hour := time.hour(time.now_ns())
    user_patterns := data.users[input.user_id].activity_patterns
    typical_hours := user_patterns.typical_hours
    
    score := case current_hour {
        current_hour in typical_hours := 0
        (current_hour >= 22) or (current_hour <= 6) := 20
        default := 5
    }
}

new_recipient_risk := score if {
    recipient_id := input.recipient_id
    user_history := data.users[input.user_id].recipients
    
    score := case recipient_id {
        recipient_id in user_history := 0
        default := 15
    }
}

geographic_risk := score if {
    user_location := input.location
    user_profile := data.users[input.user_id]
    typical_locations := user_profile.typical_locations
    
    location_match := [loc | 
        loc := typical_locations[_]
        distance(user_location, loc) < 100 # 100km radius
    ]
    
    score := case count(location_match) {
        count(location_match) > 0 := 0
        default := 25
    }
}

device_risk := score if {
    device_id := input.device_id
    user_devices := data.users[input.user_id].trusted_devices
    
    score := case device_id {
        device_id in user_devices := 0
        default := 20
    }
}

# Helper function for geographic distance calculation
distance(loc1, loc2) := dist if {
    # Simplified distance calculation (in production, use proper geospatial functions)
    lat_diff := abs(loc1.latitude - loc2.latitude)
    lon_diff := abs(loc1.longitude - loc2.longitude)
    dist := sqrt((lat_diff * lat_diff) + (lon_diff * lon_diff)) * 111 # Rough km conversion
}

# Decision rationale for audit trails
rationale := reasons if {
    allow
    reasons := [
        "transaction approved",
        sprintf("daily spend: %v/%v %s", [
            calculate_daily_spend(input.user_id, input.currency),
            get_user_limits(input.user_id).daily_limits[input.currency],
            input.currency
        ]),
        sprintf("transaction limit: %v/%v %s", [
            input.amount,
            get_user_limits(input.user_id).max_transaction[input.currency],
            input.currency
        ]),
        sprintf("balance sufficient: %v %s available", [
            get_user_balance(input.user_id, input.currency),
            input.currency
        ]),
        sprintf("risk score: %v/%v", [
            sum([factor.score | factor := calculate_risk_factors[_]]),
            get_user_limits(input.user_id).max_risk_score
        ])
    ]
}

rationale := reasons if {
    not allow
    reasons := denial_reasons
}

denial_reasons := reasons if {
    not valid_input
    reasons := ["invalid input parameters"]
}

denial_reasons := reasons if {
    valid_input
    not within_daily_limit
    daily_spent := calculate_daily_spend(input.user_id, input.currency)
    daily_limit := get_user_limits(input.user_id).daily_limits[input.currency]
    reasons := [sprintf("daily limit exceeded: %v + %v > %v %s", [
        daily_spent, input.amount, daily_limit, input.currency
    ])]
}

denial_reasons := reasons if {
    valid_input
    within_daily_limit
    not within_transaction_limit
    tx_limit := get_user_limits(input.user_id).max_transaction[input.currency]
    reasons := [sprintf("transaction limit exceeded: %v > %v %s", [
        input.amount, tx_limit, input.currency
    ])]
}

denial_reasons := reasons if {
    valid_input
    within_daily_limit
    within_transaction_limit
    not sufficient_balance
    balance := get_user_balance(input.user_id, input.currency)
    required := input.amount + calculate_fees(input.amount, input.currency)
    reasons := [sprintf("insufficient balance: %v < %v %s required", [
        balance, required, input.currency
    ])]
}

denial_reasons := reasons if {
    valid_input
    within_daily_limit
    within_transaction_limit
    sufficient_balance
    not within_velocity_limits
    recent_count := count(get_recent_transactions(input.user_id, 300))
    velocity_limit := get_user_limits(input.user_id).velocity_limits.transactions_per_5min
    reasons := [sprintf("velocity limit exceeded: %v >= %v transactions in 5 minutes", [
        recent_count, velocity_limit
    ])]
}

denial_reasons := reasons if {
    valid_input
    within_daily_limit
    within_transaction_limit
    sufficient_balance
    within_velocity_limits
    not risk_score_acceptable
    risk_score := sum([factor.score | factor := calculate_risk_factors[_]])
    max_risk := get_user_limits(input.user_id).max_risk_score
    reasons := [sprintf("risk score too high: %v > %v", [risk_score, max_risk])]
}

denial_reasons := reasons if {
    blacklisted_user
    reasons := ["user is blacklisted"]
}

denial_reasons := ["policy violation"] if {
    # Catch-all for other denial cases
    valid_input
    not blacklisted_user
    within_daily_limit
    within_transaction_limit
    sufficient_balance
    within_velocity_limits
    risk_score_acceptable
}

# Emergency override capability
emergency_override if {
    input.emergency_token
    input.admin_user_id
    verify_emergency_token(input.emergency_token, input.admin_user_id)
}

verify_emergency_token(token, admin_id) if {
    # In production, verify emergency token cryptographically
    admin_data := data.admins[admin_id]
    admin_data.emergency_access == true
    startswith(token, "emergency_")
}

# Override decision for emergencies
allow if {
    emergency_override
}

rationale := ["emergency override applied"] if {
    emergency_override
}
