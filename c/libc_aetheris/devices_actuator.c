/**
 * @file devices_actuator.c
 * @brief C FFI implementation for Aetheris Actuator Device Runtime
 * 
 * This file provides the C implementation for actuator operations,
 * including PWM outputs, relays, motor control, LEDs, and
 * deterministic state management with NGFS snapshot integration.
 */

#include "devices_actuator.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Forward declarations for Rust FFI functions
extern int32_t rust_actuator_init(void);
extern int32_t rust_actuator_shutdown(void);
extern int32_t rust_actuator_configure(
    const char* session_id,
    const char* caps,
    const char* name,
    uint32_t type,
    uint32_t pin,
    float min_value,
    float max_value,
    float default_value,
    uint32_t frequency,
    uint32_t resolution,
    bool enable_safety_limits,
    float safety_min,
    float safety_max,
    uint32_t ramp_time_ms,
    bool enable_ramping
);
extern int32_t rust_actuator_enable(
    const char* session_id,
    const char* caps,
    const char* actuator_name
);
extern int32_t rust_actuator_disable(
    const char* session_id,
    const char* caps,
    const char* actuator_name
);
extern int32_t rust_actuator_set_value(
    const char* session_id,
    const char* caps,
    const char* actuator_name,
    float value
);
extern int32_t rust_actuator_get_value(
    const char* actuator_name,
    float* value
);
extern int32_t rust_actuator_set_pattern(
    const char* session_id,
    const char* caps,
    const char* actuator_name,
    const char* pattern_name,
    const float* values,
    const uint32_t* durations,
    uint32_t step_count,
    bool loop,
    uint32_t loop_count
);
extern int32_t rust_actuator_stop_pattern(
    const char* session_id,
    const char* caps,
    const char* actuator_name
);
extern int32_t rust_actuator_get_state(
    const char* actuator_name,
    bool* configured,
    bool* enabled,
    float* current_value,
    float* target_value,
    uint64_t* last_update,
    uint64_t* update_count,
    bool* safety_enabled,
    float* safety_min,
    float* safety_max,
    bool* ramping_enabled,
    uint32_t* ramp_time_ms,
    bool* deterministic
);
extern int32_t rust_actuator_get_configured_actuators(
    char** actuator_names,
    size_t* count
);
extern int32_t rust_actuator_get_operation_history(
    size_t limit,
    char** operation_ids,
    char** actuator_names,
    uint32_t** operations,
    bool** results,
    uint64_t** timestamps,
    bool** deterministic,
    size_t* count
);
extern int32_t rust_actuator_enable_deterministic_mode(void);
extern int32_t rust_actuator_disable_deterministic_mode(void);
extern int32_t rust_actuator_advance_tick(void);
extern int32_t rust_actuator_get_tick_counter(uint64_t* tick_count);
extern int32_t rust_actuator_create_snapshot(
    uint8_t** snapshot_data,
    size_t* snapshot_size
);
extern int32_t rust_actuator_restore_snapshot(
    const uint8_t* snapshot_data,
    size_t snapshot_size
);
extern int32_t rust_actuator_set_dao_policy(
    const char* room_id,
    bool allow_configure,
    bool allow_control
);
extern void rust_actuator_free_actuator_array(char* actuator_names);
extern void rust_actuator_free_operation_arrays(
    char* operation_ids,
    char* actuator_names,
    uint32_t* operations,
    bool* results,
    uint64_t* timestamps,
    bool* deterministic
);
extern const char* rust_actuator_error_message(int32_t error);

// Helper function to convert error codes
static aetheris_actuator_error_t convert_error(int32_t rust_error) {
    switch (rust_error) {
        case 0: return AETHERIS_ACTUATOR_SUCCESS;
        case -1: return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
        case -2: return AETHERIS_ACTUATOR_ERROR_ACTUATOR_NOT_CONFIGURED;
        case -3: return AETHERIS_ACTUATOR_ERROR_ACTUATOR_DISABLED;
        case -4: return AETHERIS_ACTUATOR_ERROR_INVALID_CAPABILITY;
        case -5: return AETHERIS_ACTUATOR_ERROR_DAO_POLICY_DENIED;
        case -6: return AETHERIS_ACTUATOR_ERROR_VALUE_OUT_OF_RANGE;
        case -7: return AETHERIS_ACTUATOR_ERROR_SAFETY_VIOLATION;
        case -8: return AETHERIS_ACTUATOR_ERROR_PATTERN_NOT_FOUND;
        case -9: return AETHERIS_ACTUATOR_ERROR_OPERATION_FAILED;
        case -10: return AETHERIS_ACTUATOR_ERROR_SERIALIZATION_ERROR;
        case -11: return AETHERIS_ACTUATOR_ERROR_DESERIALIZATION_ERROR;
        default: return AETHERIS_ACTUATOR_ERROR_UNKNOWN;
    }
}

aetheris_actuator_error_t aetheris_actuator_init(void) {
    return convert_error(rust_actuator_init());
}

aetheris_actuator_error_t aetheris_actuator_shutdown(void) {
    return convert_error(rust_actuator_shutdown());
}

aetheris_actuator_error_t aetheris_actuator_configure(
    const char* session_id,
    const char* caps,
    const aetheris_actuator_config_t* config
) {
    if (!session_id || !caps || !config) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_actuator_configure(
        session_id,
        caps,
        config->name,
        (uint32_t)config->type,
        config->pin,
        config->min_value,
        config->max_value,
        config->default_value,
        config->frequency,
        config->resolution,
        config->enable_safety_limits,
        config->safety_min,
        config->safety_max,
        config->ramp_time_ms,
        config->enable_ramping
    ));
}

aetheris_actuator_error_t aetheris_actuator_enable(
    const char* session_id,
    const char* caps,
    const char* actuator_name
) {
    if (!session_id || !caps || !actuator_name) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_actuator_enable(session_id, caps, actuator_name));
}

aetheris_actuator_error_t aetheris_actuator_disable(
    const char* session_id,
    const char* caps,
    const char* actuator_name
) {
    if (!session_id || !caps || !actuator_name) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_actuator_disable(session_id, caps, actuator_name));
}

aetheris_actuator_error_t aetheris_actuator_set_value(
    const char* session_id,
    const char* caps,
    const char* actuator_name,
    float value
) {
    if (!session_id || !caps || !actuator_name) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_actuator_set_value(session_id, caps, actuator_name, value));
}

aetheris_actuator_error_t aetheris_actuator_get_value(
    const char* actuator_name,
    float* value
) {
    if (!actuator_name || !value) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_actuator_get_value(actuator_name, value));
}

aetheris_actuator_error_t aetheris_actuator_set_pattern(
    const char* session_id,
    const char* caps,
    const char* actuator_name,
    const aetheris_actuator_pattern_t* pattern
) {
    if (!session_id || !caps || !actuator_name || !pattern) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    if (pattern->step_count == 0 || !pattern->steps) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    // Extract values and durations from pattern steps
    float* values = malloc(pattern->step_count * sizeof(float));
    uint32_t* durations = malloc(pattern->step_count * sizeof(uint32_t));
    
    if (!values || !durations) {
        free(values);
        free(durations);
        return AETHERIS_ACTUATOR_ERROR_UNKNOWN;
    }

    for (uint32_t i = 0; i < pattern->step_count; i++) {
        values[i] = pattern->steps[i].value;
        durations[i] = pattern->steps[i].duration_ms;
    }

    int32_t result = rust_actuator_set_pattern(
        session_id, caps, actuator_name, pattern->name, values, durations,
        pattern->step_count, pattern->loop, pattern->loop_count
    );

    free(values);
    free(durations);

    return convert_error(result);
}

aetheris_actuator_error_t aetheris_actuator_stop_pattern(
    const char* session_id,
    const char* caps,
    const char* actuator_name
) {
    if (!session_id || !caps || !actuator_name) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_actuator_stop_pattern(session_id, caps, actuator_name));
}

aetheris_actuator_error_t aetheris_actuator_get_state(
    const char* actuator_name,
    aetheris_actuator_state_t* state
) {
    if (!actuator_name || !state) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    bool configured, enabled, safety_enabled, ramping_enabled, deterministic;
    float current_value, target_value, safety_min, safety_max;
    uint64_t last_update, update_count;
    uint32_t ramp_time_ms;

    int32_t result = rust_actuator_get_state(
        actuator_name, &configured, &enabled, &current_value, &target_value,
        &last_update, &update_count, &safety_enabled, &safety_min, &safety_max,
        &ramping_enabled, &ramp_time_ms, &deterministic
    );

    if (result == 0) {
        strncpy(state->name, actuator_name, 63);
        state->name[63] = '\0';
        state->configured = configured;
        state->enabled = enabled;
        state->current_value = current_value;
        state->target_value = target_value;
        state->last_update = last_update;
        state->update_count = update_count;
        state->safety_enabled = safety_enabled;
        state->safety_min = safety_min;
        state->safety_max = safety_max;
        state->ramping_enabled = ramping_enabled;
        state->ramp_time_ms = ramp_time_ms;
        state->deterministic = deterministic;
    }

    return convert_error(result);
}

aetheris_actuator_error_t aetheris_actuator_get_configured_actuators(
    aetheris_actuator_state_t** states,
    size_t* count
) {
    if (!states || !count) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    char* actuator_names = NULL;
    size_t actuator_count = 0;

    int32_t result = rust_actuator_get_configured_actuators(&actuator_names, &actuator_count);
    if (result != 0) {
        return convert_error(result);
    }

    if (actuator_count == 0) {
        *states = NULL;
        *count = 0;
        return AETHERIS_ACTUATOR_SUCCESS;
    }

    // Allocate memory for actuator states
    aetheris_actuator_state_t* actuator_states = malloc(actuator_count * sizeof(aetheris_actuator_state_t));
    if (!actuator_states) {
        rust_actuator_free_actuator_array(actuator_names);
        return AETHERIS_ACTUATOR_ERROR_UNKNOWN;
    }

    // Get state for each actuator
    for (size_t i = 0; i < actuator_count; i++) {
        const char* name = &actuator_names[i * 64];
        aetheris_actuator_error_t state_result = aetheris_actuator_get_state(name, &actuator_states[i]);
        if (state_result != AETHERIS_ACTUATOR_SUCCESS) {
            free(actuator_states);
            rust_actuator_free_actuator_array(actuator_names);
            return state_result;
        }
    }

    rust_actuator_free_actuator_array(actuator_names);
    *states = actuator_states;
    *count = actuator_count;
    return AETHERIS_ACTUATOR_SUCCESS;
}

void aetheris_actuator_free_actuator_states(aetheris_actuator_state_t* states) {
    if (states) {
        free(states);
    }
}

aetheris_actuator_error_t aetheris_actuator_get_operation_history(
    size_t limit,
    aetheris_actuator_operation_result_t** operations,
    size_t* count
) {
    if (!operations || !count) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    char* operation_ids = NULL;
    char* actuator_names = NULL;
    uint32_t* operation_types = NULL;
    bool* results = NULL;
    uint64_t* timestamps = NULL;
    bool* deterministic = NULL;
    size_t operation_count = 0;

    int32_t result = rust_actuator_get_operation_history(
        limit,
        &operation_ids,
        &actuator_names,
        &operation_types,
        &results,
        &timestamps,
        &deterministic,
        &operation_count
    );

    if (result != 0) {
        return convert_error(result);
    }

    if (operation_count == 0) {
        *operations = NULL;
        *count = 0;
        return AETHERIS_ACTUATOR_SUCCESS;
    }

    // Allocate memory for operation results
    aetheris_actuator_operation_result_t* operation_results = malloc(
        operation_count * sizeof(aetheris_actuator_operation_result_t)
    );
    if (!operation_results) {
        rust_actuator_free_operation_arrays(
            operation_ids, actuator_names, operation_types, results, timestamps, deterministic
        );
        return AETHERIS_ACTUATOR_ERROR_UNKNOWN;
    }

    // Copy data from Rust arrays
    for (size_t i = 0; i < operation_count; i++) {
        strncpy(operation_results[i].operation_id, &operation_ids[i * 64], 63);
        operation_results[i].operation_id[63] = '\0';
        strncpy(operation_results[i].actuator_name, &actuator_names[i * 64], 63);
        operation_results[i].actuator_name[63] = '\0';
        operation_results[i].operation = (aetheris_actuator_operation_t)operation_types[i];
        operation_results[i].result = results[i];
        operation_results[i].timestamp = timestamps[i];
        operation_results[i].deterministic = deterministic[i];
    }

    rust_actuator_free_operation_arrays(
        operation_ids, actuator_names, operation_types, results, timestamps, deterministic
    );

    *operations = operation_results;
    *count = operation_count;
    return AETHERIS_ACTUATOR_SUCCESS;
}

void aetheris_actuator_free_operation_history(aetheris_actuator_operation_result_t* operations) {
    if (operations) {
        free(operations);
    }
}

aetheris_actuator_error_t aetheris_actuator_enable_deterministic_mode(void) {
    return convert_error(rust_actuator_enable_deterministic_mode());
}

aetheris_actuator_error_t aetheris_actuator_disable_deterministic_mode(void) {
    return convert_error(rust_actuator_disable_deterministic_mode());
}

aetheris_actuator_error_t aetheris_actuator_advance_tick(void) {
    return convert_error(rust_actuator_advance_tick());
}

aetheris_actuator_error_t aetheris_actuator_get_tick_counter(uint64_t* tick_count) {
    if (!tick_count) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_actuator_get_tick_counter(tick_count));
}

aetheris_actuator_error_t aetheris_actuator_create_snapshot(
    uint8_t** snapshot_data,
    size_t* snapshot_size
) {
    if (!snapshot_data || !snapshot_size) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_actuator_create_snapshot(snapshot_data, snapshot_size));
}

aetheris_actuator_error_t aetheris_actuator_restore_snapshot(
    const uint8_t* snapshot_data,
    size_t snapshot_size
) {
    if (!snapshot_data || snapshot_size == 0) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_actuator_restore_snapshot(snapshot_data, snapshot_size));
}

void aetheris_actuator_free_snapshot(uint8_t* snapshot_data) {
    if (snapshot_data) {
        free(snapshot_data);
    }
}

const char* aetheris_actuator_error_message(aetheris_actuator_error_t error) {
    return rust_actuator_error_message((int32_t)error);
}

void aetheris_actuator_config_default(aetheris_actuator_config_t* config) {
    if (!config) {
        return;
    }

    strcpy(config->name, "actuator");
    config->type = AETHERIS_ACTUATOR_TYPE_PWM;
    config->pin = 0;
    config->min_value = 0.0f;
    config->max_value = 1.0f;
    config->default_value = 0.0f;
    config->frequency = 1000;
    config->resolution = 8;
    config->enable_safety_limits = true;
    config->safety_min = 0.0f;
    config->safety_max = 1.0f;
    config->ramp_time_ms = 100;
    config->enable_ramping = false;
}

aetheris_actuator_error_t aetheris_actuator_set_dao_policy(
    const char* room_id,
    bool allow_configure,
    bool allow_control
) {
    if (!room_id) {
        return AETHERIS_ACTUATOR_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_actuator_set_dao_policy(room_id, allow_configure, allow_control));
}
