/**
 * @file devices_adc.c
 * @brief C FFI implementation for Aetheris ADC Device Runtime
 * 
 * This file provides the C implementation for ADC operations,
 * including channel configuration, analog sampling, and
 * deterministic state management with NGFS snapshot integration.
 */

#include "devices_adc.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Forward declarations for Rust FFI functions
extern int32_t rust_adc_init(void);
extern int32_t rust_adc_shutdown(void);
extern int32_t rust_adc_configure_channel(
    const char* session_id,
    const char* caps,
    uint32_t channel,
    uint32_t sample_rate,
    uint32_t resolution,
    float reference_voltage,
    bool enable_calibration,
    float calibration_offset,
    float calibration_scale,
    uint32_t oversampling,
    bool enable_filtering,
    float filter_cutoff
);
extern int32_t rust_adc_enable_channel(
    const char* session_id,
    const char* caps,
    uint32_t channel
);
extern int32_t rust_adc_disable_channel(
    const char* session_id,
    const char* caps,
    uint32_t channel
);
extern int32_t rust_adc_sample_channel(
    const char* session_id,
    const char* caps,
    uint32_t channel,
    float* value,
    float* raw_value,
    uint64_t* timestamp,
    bool* deterministic,
    uint64_t* tick_count
);
extern int32_t rust_adc_sample_channels(
    const char* session_id,
    const char* caps,
    const uint32_t* channels,
    size_t channel_count,
    float* values,
    float* raw_values,
    uint64_t* timestamps,
    bool* deterministic,
    uint64_t* tick_counts
);
extern int32_t rust_adc_get_channel_state(
    uint32_t channel,
    bool* configured,
    bool* enabled,
    uint32_t* sample_rate,
    uint32_t* resolution,
    float* reference_voltage,
    float* last_sample,
    uint64_t* last_sample_time,
    uint64_t* sample_count,
    float* min_value,
    float* max_value,
    float* average_value,
    bool* calibration_enabled,
    float* calibration_offset,
    float* calibration_scale
);
extern int32_t rust_adc_get_configured_channels(
    uint32_t** channels,
    size_t* count
);
extern int32_t rust_adc_get_operation_history(
    size_t limit,
    char** operation_ids,
    uint32_t** channels,
    uint32_t** operations,
    bool** results,
    uint64_t** timestamps,
    bool** deterministic,
    size_t* count
);
extern int32_t rust_adc_enable_deterministic_mode(void);
extern int32_t rust_adc_disable_deterministic_mode(void);
extern int32_t rust_adc_advance_tick(void);
extern int32_t rust_adc_get_tick_counter(uint64_t* tick_count);
extern int32_t rust_adc_create_snapshot(
    uint8_t** snapshot_data,
    size_t* snapshot_size
);
extern int32_t rust_adc_restore_snapshot(
    const uint8_t* snapshot_data,
    size_t snapshot_size
);
extern int32_t rust_adc_set_dao_policy(
    const char* room_id,
    bool allow_configure,
    bool allow_sample
);
extern void rust_adc_free_channel_array(uint32_t* channels);
extern void rust_adc_free_operation_arrays(
    char* operation_ids,
    uint32_t* channels,
    uint32_t* operations,
    bool* results,
    uint64_t* timestamps,
    bool* deterministic
);
extern const char* rust_adc_error_message(int32_t error);

// Helper function to convert error codes
static aetheris_adc_error_t convert_error(int32_t rust_error) {
    switch (rust_error) {
        case 0: return AETHERIS_ADC_SUCCESS;
        case -1: return AETHERIS_ADC_ERROR_INVALID_PARAM;
        case -2: return AETHERIS_ADC_ERROR_CHANNEL_NOT_CONFIGURED;
        case -3: return AETHERIS_ADC_ERROR_CHANNEL_DISABLED;
        case -4: return AETHERIS_ADC_ERROR_INVALID_CAPABILITY;
        case -5: return AETHERIS_ADC_ERROR_DAO_POLICY_DENIED;
        case -6: return AETHERIS_ADC_ERROR_SAMPLING_FAILED;
        case -7: return AETHERIS_ADC_ERROR_SERIALIZATION_ERROR;
        case -8: return AETHERIS_ADC_ERROR_DESERIALIZATION_ERROR;
        default: return AETHERIS_ADC_ERROR_UNKNOWN;
    }
}

aetheris_adc_error_t aetheris_adc_init(void) {
    return convert_error(rust_adc_init());
}

aetheris_adc_error_t aetheris_adc_shutdown(void) {
    return convert_error(rust_adc_shutdown());
}

aetheris_adc_error_t aetheris_adc_configure_channel(
    const char* session_id,
    const char* caps,
    const aetheris_adc_config_t* config
) {
    if (!session_id || !caps || !config) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_adc_configure_channel(
        session_id,
        caps,
        config->channel,
        config->sample_rate,
        config->resolution,
        config->reference_voltage,
        config->enable_calibration,
        config->calibration_offset,
        config->calibration_scale,
        config->oversampling,
        config->enable_filtering,
        config->filter_cutoff
    ));
}

aetheris_adc_error_t aetheris_adc_enable_channel(
    const char* session_id,
    const char* caps,
    uint32_t channel
) {
    if (!session_id || !caps) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_adc_enable_channel(session_id, caps, channel));
}

aetheris_adc_error_t aetheris_adc_disable_channel(
    const char* session_id,
    const char* caps,
    uint32_t channel
) {
    if (!session_id || !caps) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_adc_disable_channel(session_id, caps, channel));
}

aetheris_adc_error_t aetheris_adc_sample_channel(
    const char* session_id,
    const char* caps,
    uint32_t channel,
    aetheris_adc_sample_t* sample
) {
    if (!session_id || !caps || !sample) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    float value, raw_value;
    uint64_t timestamp, tick_count;
    bool deterministic;

    int32_t result = rust_adc_sample_channel(
        session_id, caps, channel, &value, &raw_value, &timestamp, &deterministic, &tick_count
    );

    if (result == 0) {
        sample->channel = channel;
        sample->value = value;
        sample->raw_value = raw_value;
        sample->timestamp = timestamp;
        sample->deterministic = deterministic;
        sample->tick_count = tick_count;
    }

    return convert_error(result);
}

aetheris_adc_error_t aetheris_adc_sample_channels(
    const char* session_id,
    const char* caps,
    const uint32_t* channels,
    size_t channel_count,
    aetheris_adc_sample_t* samples
) {
    if (!session_id || !caps || !channels || !samples || channel_count == 0) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    float* values = malloc(channel_count * sizeof(float));
    float* raw_values = malloc(channel_count * sizeof(float));
    uint64_t* timestamps = malloc(channel_count * sizeof(uint64_t));
    bool* deterministic = malloc(channel_count * sizeof(bool));
    uint64_t* tick_counts = malloc(channel_count * sizeof(uint64_t));

    if (!values || !raw_values || !timestamps || !deterministic || !tick_counts) {
        free(values);
        free(raw_values);
        free(timestamps);
        free(deterministic);
        free(tick_counts);
        return AETHERIS_ADC_ERROR_UNKNOWN;
    }

    int32_t result = rust_adc_sample_channels(
        session_id, caps, channels, channel_count, values, raw_values, timestamps, deterministic, tick_counts
    );

    if (result == 0) {
        for (size_t i = 0; i < channel_count; i++) {
            samples[i].channel = channels[i];
            samples[i].value = values[i];
            samples[i].raw_value = raw_values[i];
            samples[i].timestamp = timestamps[i];
            samples[i].deterministic = deterministic[i];
            samples[i].tick_count = tick_counts[i];
        }
    }

    free(values);
    free(raw_values);
    free(timestamps);
    free(deterministic);
    free(tick_counts);

    return convert_error(result);
}

aetheris_adc_error_t aetheris_adc_get_channel_state(
    uint32_t channel,
    aetheris_adc_state_t* state
) {
    if (!state) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    bool configured, enabled, calibration_enabled;
    uint32_t sample_rate, resolution;
    float reference_voltage, last_sample, min_value, max_value, average_value;
    float calibration_offset, calibration_scale;
    uint64_t last_sample_time, sample_count;

    int32_t result = rust_adc_get_channel_state(
        channel, &configured, &enabled, &sample_rate, &resolution, &reference_voltage,
        &last_sample, &last_sample_time, &sample_count, &min_value, &max_value,
        &average_value, &calibration_enabled, &calibration_offset, &calibration_scale
    );

    if (result == 0) {
        state->channel = channel;
        state->configured = configured;
        state->enabled = enabled;
        state->sample_rate = sample_rate;
        state->resolution = resolution;
        state->reference_voltage = reference_voltage;
        state->last_sample = last_sample;
        state->last_sample_time = last_sample_time;
        state->sample_count = sample_count;
        state->min_value = min_value;
        state->max_value = max_value;
        state->average_value = average_value;
        state->calibration_enabled = calibration_enabled;
        state->calibration_offset = calibration_offset;
        state->calibration_scale = calibration_scale;
    }

    return convert_error(result);
}

aetheris_adc_error_t aetheris_adc_get_configured_channels(
    aetheris_adc_state_t** states,
    size_t* count
) {
    if (!states || !count) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    uint32_t* channels = NULL;
    size_t channel_count = 0;

    int32_t result = rust_adc_get_configured_channels(&channels, &channel_count);
    if (result != 0) {
        return convert_error(result);
    }

    if (channel_count == 0) {
        *states = NULL;
        *count = 0;
        return AETHERIS_ADC_SUCCESS;
    }

    // Allocate memory for channel states
    aetheris_adc_state_t* channel_states = malloc(channel_count * sizeof(aetheris_adc_state_t));
    if (!channel_states) {
        rust_adc_free_channel_array(channels);
        return AETHERIS_ADC_ERROR_UNKNOWN;
    }

    // Get state for each channel
    for (size_t i = 0; i < channel_count; i++) {
        aetheris_adc_error_t state_result = aetheris_adc_get_channel_state(channels[i], &channel_states[i]);
        if (state_result != AETHERIS_ADC_SUCCESS) {
            free(channel_states);
            rust_adc_free_channel_array(channels);
            return state_result;
        }
    }

    rust_adc_free_channel_array(channels);
    *states = channel_states;
    *count = channel_count;
    return AETHERIS_ADC_SUCCESS;
}

void aetheris_adc_free_channel_states(aetheris_adc_state_t* states) {
    if (states) {
        free(states);
    }
}

aetheris_adc_error_t aetheris_adc_get_operation_history(
    size_t limit,
    aetheris_adc_operation_result_t** operations,
    size_t* count
) {
    if (!operations || !count) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    char* operation_ids = NULL;
    uint32_t* channels = NULL;
    uint32_t* operation_types = NULL;
    bool* results = NULL;
    uint64_t* timestamps = NULL;
    bool* deterministic = NULL;
    size_t operation_count = 0;

    int32_t result = rust_adc_get_operation_history(
        limit,
        &operation_ids,
        &channels,
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
        return AETHERIS_ADC_SUCCESS;
    }

    // Allocate memory for operation results
    aetheris_adc_operation_result_t* operation_results = malloc(
        operation_count * sizeof(aetheris_adc_operation_result_t)
    );
    if (!operation_results) {
        rust_adc_free_operation_arrays(
            operation_ids, channels, operation_types, results, timestamps, deterministic
        );
        return AETHERIS_ADC_ERROR_UNKNOWN;
    }

    // Copy data from Rust arrays
    for (size_t i = 0; i < operation_count; i++) {
        strncpy(operation_results[i].operation_id, &operation_ids[i * 64], 63);
        operation_results[i].operation_id[63] = '\0';
        operation_results[i].channel = channels[i];
        operation_results[i].operation = (aetheris_adc_operation_t)operation_types[i];
        operation_results[i].result = results[i];
        operation_results[i].timestamp = timestamps[i];
        operation_results[i].deterministic = deterministic[i];
    }

    rust_adc_free_operation_arrays(
        operation_ids, channels, operation_types, results, timestamps, deterministic
    );

    *operations = operation_results;
    *count = operation_count;
    return AETHERIS_ADC_SUCCESS;
}

void aetheris_adc_free_operation_history(aetheris_adc_operation_result_t* operations) {
    if (operations) {
        free(operations);
    }
}

aetheris_adc_error_t aetheris_adc_enable_deterministic_mode(void) {
    return convert_error(rust_adc_enable_deterministic_mode());
}

aetheris_adc_error_t aetheris_adc_disable_deterministic_mode(void) {
    return convert_error(rust_adc_disable_deterministic_mode());
}

aetheris_adc_error_t aetheris_adc_advance_tick(void) {
    return convert_error(rust_adc_advance_tick());
}

aetheris_adc_error_t aetheris_adc_get_tick_counter(uint64_t* tick_count) {
    if (!tick_count) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_adc_get_tick_counter(tick_count));
}

aetheris_adc_error_t aetheris_adc_create_snapshot(
    uint8_t** snapshot_data,
    size_t* snapshot_size
) {
    if (!snapshot_data || !snapshot_size) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_adc_create_snapshot(snapshot_data, snapshot_size));
}

aetheris_adc_error_t aetheris_adc_restore_snapshot(
    const uint8_t* snapshot_data,
    size_t snapshot_size
) {
    if (!snapshot_data || snapshot_size == 0) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_adc_restore_snapshot(snapshot_data, snapshot_size));
}

void aetheris_adc_free_snapshot(uint8_t* snapshot_data) {
    if (snapshot_data) {
        free(snapshot_data);
    }
}

const char* aetheris_adc_error_message(aetheris_adc_error_t error) {
    return rust_adc_error_message((int32_t)error);
}

void aetheris_adc_config_default(aetheris_adc_config_t* config) {
    if (!config) {
        return;
    }

    config->channel = 0;
    config->sample_rate = 1000;
    config->resolution = 12;
    config->reference_voltage = 3.3f;
    config->enable_calibration = false;
    config->calibration_offset = 0.0f;
    config->calibration_scale = 1.0f;
    config->oversampling = 1;
    config->enable_filtering = false;
    config->filter_cutoff = 100.0f;
}

aetheris_adc_error_t aetheris_adc_set_dao_policy(
    const char* room_id,
    bool allow_configure,
    bool allow_sample
) {
    if (!room_id) {
        return AETHERIS_ADC_ERROR_INVALID_PARAM;
    }

    return convert_error(rust_adc_set_dao_policy(room_id, allow_configure, allow_sample));
}
