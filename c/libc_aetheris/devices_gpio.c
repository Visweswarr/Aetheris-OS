/**
 * @file devices_gpio.c
 * @brief C FFI implementation for Aetheris GPIO Device Runtime
 * 
 * This file provides the C implementation for GPIO operations,
 * including pin configuration, digital I/O, and deterministic
 * state management with NGFS snapshot integration.
 */

#include "devices_gpio.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Forward declarations for Rust FFI functions
extern int32_t rust_gpio_init(void);
extern int32_t rust_gpio_shutdown(void);
extern int32_t rust_gpio_configure_pin(
    const char* session_id,
    const char* caps,
    uint32_t pin,
    uint32_t mode,
    uint32_t pull,
    bool initial_value,
    uint32_t debounce_ms
);
extern int32_t rust_gpio_read_pin(
    const char* session_id,
    const char* caps,
    uint32_t pin,
    bool* value
);
extern int32_t rust_gpio_write_pin(
    const char* session_id,
    const char* caps,
    uint32_t pin,
    bool value
);
extern int32_t rust_gpio_toggle_pin(
    const char* session_id,
    const char* caps,
    uint32_t pin,
    bool* new_value
);
extern int32_t rust_gpio_get_pin_state(
    uint32_t pin,
    uint32_t* mode,
    uint32_t* pull,
    bool* value,
    uint64_t* last_change,
    uint64_t* change_count
);
extern int32_t rust_gpio_get_configured_pins(
    uint32_t** pins,
    size_t* count
);
extern int32_t rust_gpio_get_operation_history(
    size_t limit,
    char** operation_ids,
    uint32_t** pins,
    uint32_t** operations,
    bool** results,
    uint64_t** timestamps,
    bool** deterministic,
    size_t* count
);
extern int32_t rust_gpio_enable_deterministic_mode(void);
extern int32_t rust_gpio_disable_deterministic_mode(void);
extern int32_t rust_gpio_advance_tick(void);
extern int32_t rust_gpio_get_tick_counter(uint64_t* tick_count);
extern int32_t rust_gpio_create_snapshot(
    uint8_t** snapshot_data,
    size_t* snapshot_size
);
extern int32_t rust_gpio_restore_snapshot(
    const uint8_t* snapshot_data,
    size_t snapshot_size
);
extern int32_t rust_gpio_set_dao_policy(
    const char* room_id,
    bool allow_configure,
    bool allow_read,
    bool allow_write
);
extern void rust_gpio_free_pin_array(uint32_t* pins);
extern void rust_gpio_free_operation_arrays(
    char* operation_ids,
    uint32_t* pins,
    uint32_t* operations,
    bool* results,
    uint64_t* timestamps,
    bool* deterministic
);
extern const char* rust_gpio_error_message(int32_t error);

// Helper function to convert error codes
static aetheris_gpio_error_t convert_error(int32_t rust_error) {
    switch (rust_error) {
        case 0: return AETHERIS_GPIO_SUCCESS;
        case -1: return AETHERIS_GPIO_ERROR_INVALID_PARAM;
        case -2: return AETHERIS_GPIO_ERROR_PIN_NOT_CONFIGURED;
        case -3: return AETHERIS_GPIO_ERROR_INVALID_PIN_MODE;
        case -4: return AETHERIS_GPIO_ERROR_INVALID_CAPABILITY;
        case -5: return AETHERIS_GPIO_ERROR_DAO_POLICY_DENIED;
        case -6: return AETHERIS_GPIO_ERROR_OPERATION_FAILED;
        case -7: return AETHERIS_GPIO_ERROR_SERIALIZATION_ERROR;
        case -8: return AETHERIS_GPIO_ERROR_DESERIALIZATION_ERROR;
        default: return AETHERIS_GPIO_ERROR_UNKNOWN;
    }
}

aetheris_gpio_error_t aetheris_gpio_init(void) {
    return convert_error(rust_gpio_init());
}

aetheris_gpio_error_t aetheris_gpio_shutdown(void) {
    return convert_error(rust_gpio_shutdown());
}

aetheris_gpio_error_t aetheris_gpio_configure_pin(
    const char* session_id,
    const char* caps,
    const aetheris_gpio_config_t* config
) {
    if (!session_id || !caps || !config) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_gpio_configure_pin(
        session_id,
        caps,
        config->pin,
        (uint32_t)config->mode,
        (uint32_t)config->pull,
        config->initial_value,
        config->debounce_ms
    ));
}

aetheris_gpio_error_t aetheris_gpio_read_pin(
    const char* session_id,
    const char* caps,
    uint32_t pin,
    bool* value
) {
    if (!session_id || !caps || !value) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_gpio_read_pin(session_id, caps, pin, value));
}

aetheris_gpio_error_t aetheris_gpio_write_pin(
    const char* session_id,
    const char* caps,
    uint32_t pin,
    bool value
) {
    if (!session_id || !caps) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_gpio_write_pin(session_id, caps, pin, value));
}

aetheris_gpio_error_t aetheris_gpio_toggle_pin(
    const char* session_id,
    const char* caps,
    uint32_t pin,
    bool* new_value
) {
    if (!session_id || !caps || !new_value) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_gpio_toggle_pin(session_id, caps, pin, new_value));
}

aetheris_gpio_error_t aetheris_gpio_get_pin_state(
    uint32_t pin,
    aetheris_gpio_state_t* state
) {
    if (!state) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    uint32_t mode, pull;
    bool value;
    uint64_t last_change, change_count;

    int32_t result = rust_gpio_get_pin_state(
        pin, &mode, &pull, &value, &last_change, &change_count
    );

    if (result == 0) {
        state->pin = pin;
        state->mode = (aetheris_gpio_mode_t)mode;
        state->pull = (aetheris_gpio_pull_t)pull;
        state->value = value;
        state->last_change = last_change;
        state->change_count = change_count;
    }

    return convert_error(result);
}

aetheris_gpio_error_t aetheris_gpio_get_configured_pins(
    aetheris_gpio_state_t** states,
    size_t* count
) {
    if (!states || !count) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    uint32_t* pins = NULL;
    size_t pin_count = 0;

    int32_t result = rust_gpio_get_configured_pins(&pins, &pin_count);
    if (result != 0) {
        return convert_error(result);
    }

    if (pin_count == 0) {
        *states = NULL;
        *count = 0;
        return AETHERIS_GPIO_SUCCESS;
    }

    // Allocate memory for pin states
    aetheris_gpio_state_t* pin_states = malloc(pin_count * sizeof(aetheris_gpio_state_t));
    if (!pin_states) {
        rust_gpio_free_pin_array(pins);
        return AETHERIS_GPIO_ERROR_UNKNOWN;
    }

    // Get state for each pin
    for (size_t i = 0; i < pin_count; i++) {
        aetheris_gpio_error_t state_result = aetheris_gpio_get_pin_state(pins[i], &pin_states[i]);
        if (state_result != AETHERIS_GPIO_SUCCESS) {
            free(pin_states);
            rust_gpio_free_pin_array(pins);
            return state_result;
        }
    }

    rust_gpio_free_pin_array(pins);
    *states = pin_states;
    *count = pin_count;
    return AETHERIS_GPIO_SUCCESS;
}

void aetheris_gpio_free_pin_states(aetheris_gpio_state_t* states) {
    if (states) {
        free(states);
    }
}

aetheris_gpio_error_t aetheris_gpio_get_operation_history(
    size_t limit,
    aetheris_gpio_operation_result_t** operations,
    size_t* count
) {
    if (!operations || !count) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    char* operation_ids = NULL;
    uint32_t* pins = NULL;
    uint32_t* operation_types = NULL;
    bool* results = NULL;
    uint64_t* timestamps = NULL;
    bool* deterministic = NULL;
    size_t operation_count = 0;

    int32_t result = rust_gpio_get_operation_history(
        limit,
        &operation_ids,
        &pins,
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
        return AETHERIS_GPIO_SUCCESS;
    }

    // Allocate memory for operation results
    aetheris_gpio_operation_result_t* operation_results = malloc(
        operation_count * sizeof(aetheris_gpio_operation_result_t)
    );
    if (!operation_results) {
        rust_gpio_free_operation_arrays(
            operation_ids, pins, operation_types, results, timestamps, deterministic
        );
        return AETHERIS_GPIO_ERROR_UNKNOWN;
    }

    // Copy data from Rust arrays
    for (size_t i = 0; i < operation_count; i++) {
        strncpy(operation_results[i].operation_id, &operation_ids[i * 64], 63);
        operation_results[i].operation_id[63] = '\0';
        operation_results[i].pin = pins[i];
        operation_results[i].operation = (aetheris_gpio_operation_t)operation_types[i];
        operation_results[i].result = results[i];
        operation_results[i].timestamp = timestamps[i];
        operation_results[i].deterministic = deterministic[i];
    }

    rust_gpio_free_operation_arrays(
        operation_ids, pins, operation_types, results, timestamps, deterministic
    );

    *operations = operation_results;
    *count = operation_count;
    return AETHERIS_GPIO_SUCCESS;
}

void aetheris_gpio_free_operation_history(aetheris_gpio_operation_result_t* operations) {
    if (operations) {
        free(operations);
    }
}

aetheris_gpio_error_t aetheris_gpio_enable_deterministic_mode(void) {
    return convert_error(rust_gpio_enable_deterministic_mode());
}

aetheris_gpio_error_t aetheris_gpio_disable_deterministic_mode(void) {
    return convert_error(rust_gpio_disable_deterministic_mode());
}

aetheris_gpio_error_t aetheris_gpio_advance_tick(void) {
    return convert_error(rust_gpio_advance_tick());
}

aetheris_gpio_error_t aetheris_gpio_get_tick_counter(uint64_t* tick_count) {
    if (!tick_count) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_gpio_get_tick_counter(tick_count));
}

aetheris_gpio_error_t aetheris_gpio_create_snapshot(
    uint8_t** snapshot_data,
    size_t* snapshot_size
) {
    if (!snapshot_data || !snapshot_size) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_gpio_create_snapshot(snapshot_data, snapshot_size));
}

aetheris_gpio_error_t aetheris_gpio_restore_snapshot(
    const uint8_t* snapshot_data,
    size_t snapshot_size
) {
    if (!snapshot_data || snapshot_size == 0) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_gpio_restore_snapshot(snapshot_data, snapshot_size));
}

void aetheris_gpio_free_snapshot(uint8_t* snapshot_data) {
    if (snapshot_data) {
        free(snapshot_data);
    }
}

const char* aetheris_gpio_error_message(aetheris_gpio_error_t error) {
    return rust_gpio_error_message((int32_t)error);
}

void aetheris_gpio_config_default(aetheris_gpio_config_t* config) {
    if (!config) {
        return;
    }

    config->pin = 0;
    config->mode = AETHERIS_GPIO_MODE_INPUT;
    config->pull = AETHERIS_GPIO_PULL_NONE;
    config->initial_value = false;
    config->debounce_ms = 0;
}

aetheris_gpio_error_t aetheris_gpio_set_dao_policy(
    const char* room_id,
    bool allow_configure,
    bool allow_read,
    bool allow_write
) {
    if (!room_id) {
        return AETHERIS_GPIO_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_gpio_set_dao_policy(room_id, allow_configure, allow_read, allow_write));
}
