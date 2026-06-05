/**
 * @file devices_sensor.c
 * @brief C FFI implementation for Aetheris Sensor Device Runtime
 * 
 * This file provides the C implementation for sensor operations,
 * acting as a thin wrapper around the Rust sensor runtime.
 */

#include "devices_sensor.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <time.h>

// Mock implementation for demonstration
// In a real implementation, this would call into the Rust sensor runtime via FFI

static bool sensor_runtime_initialized = false;
static aetheris_sensor_info_t mock_sensors[10];
static size_t mock_sensor_count = 0;

aetheris_sensor_error_t aetheris_sensor_init(void) {
    if (sensor_runtime_initialized) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    // TODO: Initialize Rust sensor runtime
    sensor_runtime_initialized = true;
    mock_sensor_count = 0;
    
    return AETHERIS_SENSOR_SUCCESS;
}

aetheris_sensor_error_t aetheris_sensor_shutdown(void) {
    if (!sensor_runtime_initialized) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    // TODO: Shutdown Rust sensor runtime
    sensor_runtime_initialized = false;
    mock_sensor_count = 0;
    
    return AETHERIS_SENSOR_SUCCESS;
}

aetheris_sensor_error_t aetheris_sensor_register(
    const aetheris_sensor_desc_t* desc,
    char* sensor_id
) {
    if (!sensor_runtime_initialized) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    if (!desc || !sensor_id) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    if (mock_sensor_count >= 10) {
        return AETHERIS_SENSOR_ERROR_REGISTRATION_FAILED;
    }
    
    // Mock implementation - generate a sensor ID
    snprintf(sensor_id, 64, "sensor_%ld", (long)time(NULL));
    
    // Add to mock sensors list
    aetheris_sensor_info_t* sensor = &mock_sensors[mock_sensor_count];
    strncpy(sensor->sensor_id, sensor_id, sizeof(sensor->sensor_id) - 1);
    sensor->sensor_id[sizeof(sensor->sensor_id) - 1] = '\0';
    
    strncpy(sensor->name, desc->name, sizeof(sensor->name) - 1);
    sensor->name[sizeof(sensor->name) - 1] = '\0';
    
    sensor->kind = desc->kind;
    
    strncpy(sensor->location, desc->location, sizeof(sensor->location) - 1);
    sensor->location[sizeof(sensor->location) - 1] = '\0';
    
    strncpy(sensor->unit, desc->unit, sizeof(sensor->unit) - 1);
    sensor->unit[sizeof(sensor->unit) - 1] = '\0';
    
    sensor->range_min = desc->range_min;
    sensor->range_max = desc->range_max;
    sensor->resolution = desc->resolution;
    
    sensor->sample_rate_count = desc->sample_rate_count;
    for (uint8_t i = 0; i < desc->sample_rate_count && i < 8; i++) {
        sensor->sample_rates[i] = desc->sample_rates[i];
    }
    
    sensor->registered_at_timestamp = (uint64_t)time(NULL);
    sensor->active_sampling = false;
    
    mock_sensor_count++;
    
    // TODO: Call Rust sensor runtime
    // sensor_runtime_register(desc, sensor_id);
    
    return AETHERIS_SENSOR_SUCCESS;
}

aetheris_sensor_error_t aetheris_sensor_list(
    aetheris_sensor_info_t* sensors,
    size_t max_sensors,
    size_t* sensor_count
) {
    if (!sensor_runtime_initialized) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    if (!sensors || !sensor_count) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    size_t count = (mock_sensor_count < max_sensors) ? mock_sensor_count : max_sensors;
    
    for (size_t i = 0; i < count; i++) {
        sensors[i] = mock_sensors[i];
    }
    
    *sensor_count = count;
    
    // TODO: Call Rust sensor runtime
    // sensor_runtime_list(sensors, max_sensors, sensor_count);
    
    return AETHERIS_SENSOR_SUCCESS;
}

aetheris_sensor_error_t aetheris_sensor_start_sampling(
    const char* session_id,
    const char* caps,
    const char* sensor_id,
    uint32_t hz,
    uint64_t seed,
    char* handle
) {
    if (!sensor_runtime_initialized) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    if (!session_id || !caps || !sensor_id || !handle) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    // Check capability
    if (!strstr(caps, "device:sensor.sample")) {
        return AETHERIS_SENSOR_ERROR_INVALID_CAPABILITY;
    }
    
    // Find sensor
    bool sensor_found = false;
    for (size_t i = 0; i < mock_sensor_count; i++) {
        if (strcmp(mock_sensors[i].sensor_id, sensor_id) == 0) {
            sensor_found = true;
            
            // Check if sample rate is supported
            bool rate_supported = false;
            for (uint8_t j = 0; j < mock_sensors[i].sample_rate_count; j++) {
                if (mock_sensors[i].sample_rates[j] == hz) {
                    rate_supported = true;
                    break;
                }
            }
            
            if (!rate_supported) {
                return AETHERIS_SENSOR_ERROR_INVALID_SAMPLE_RATE;
            }
            
            // Mark as active
            mock_sensors[i].active_sampling = true;
            break;
        }
    }
    
    if (!sensor_found) {
        return AETHERIS_SENSOR_ERROR_DEVICE_NOT_FOUND;
    }
    
    // Mock implementation - generate a handle
    snprintf(handle, 64, "sampling_%ld", (long)time(NULL));
    
    // TODO: Call Rust sensor runtime
    // sensor_runtime_start_sampling(session_id, caps, sensor_id, hz, seed, handle);
    
    return AETHERIS_SENSOR_SUCCESS;
}

aetheris_sensor_error_t aetheris_sensor_stop_sampling(const char* handle) {
    if (!sensor_runtime_initialized) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    if (!handle) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    // Mock implementation - mark all sensors as inactive
    for (size_t i = 0; i < mock_sensor_count; i++) {
        mock_sensors[i].active_sampling = false;
    }
    
    // TODO: Call Rust sensor runtime
    // sensor_runtime_stop_sampling(handle);
    
    return AETHERIS_SENSOR_SUCCESS;
}

aetheris_sensor_error_t aetheris_sensor_preview(
    const char* sensor_id,
    uint32_t hz_max,
    char* stream_handle
) {
    if (!sensor_runtime_initialized) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    if (!sensor_id || !stream_handle) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    // Find sensor
    bool sensor_found = false;
    for (size_t i = 0; i < mock_sensor_count; i++) {
        if (strcmp(mock_sensors[i].sensor_id, sensor_id) == 0) {
            sensor_found = true;
            break;
        }
    }
    
    if (!sensor_found) {
        return AETHERIS_SENSOR_ERROR_DEVICE_NOT_FOUND;
    }
    
    // Mock implementation - generate a stream handle
    snprintf(stream_handle, 64, "preview_%ld", (long)time(NULL));
    
    // TODO: Call Rust sensor runtime
    // sensor_runtime_preview(sensor_id, hz_max, stream_handle);
    
    return AETHERIS_SENSOR_SUCCESS;
}

aetheris_sensor_error_t aetheris_sensor_snapshot(
    const char* sensor_id,
    const char* out_ngfs_path,
    char* snapshot_id
) {
    if (!sensor_runtime_initialized) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    if (!sensor_id || !out_ngfs_path || !snapshot_id) {
        return AETHERIS_SENSOR_ERROR_INVALID_PARAM;
    }
    
    // Find sensor
    bool sensor_found = false;
    for (size_t i = 0; i < mock_sensor_count; i++) {
        if (strcmp(mock_sensors[i].sensor_id, sensor_id) == 0) {
            sensor_found = true;
            break;
        }
    }
    
    if (!sensor_found) {
        return AETHERIS_SENSOR_ERROR_DEVICE_NOT_FOUND;
    }
    
    // Mock implementation - generate a snapshot ID
    snprintf(snapshot_id, 64, "snapshot_%ld", (long)time(NULL));
    
    // TODO: Call Rust sensor runtime
    // sensor_runtime_snapshot(sensor_id, out_ngfs_path, snapshot_id);
    
    return AETHERIS_SENSOR_SUCCESS;
}

const char* aetheris_sensor_error_message(aetheris_sensor_error_t error) {
    switch (error) {
        case AETHERIS_SENSOR_SUCCESS:
            return "Success";
        case AETHERIS_SENSOR_ERROR_INVALID_PARAM:
            return "Invalid parameter";
        case AETHERIS_SENSOR_ERROR_DEVICE_NOT_FOUND:
            return "Device not found";
        case AETHERIS_SENSOR_ERROR_REGISTRATION_FAILED:
            return "Sensor registration failed";
        case AETHERIS_SENSOR_ERROR_SAMPLING_FAILED:
            return "Sensor sampling failed";
        case AETHERIS_SENSOR_ERROR_PREVIEW_FAILED:
            return "Sensor preview failed";
        case AETHERIS_SENSOR_ERROR_SNAPSHOT_FAILED:
            return "Sensor snapshot failed";
        case AETHERIS_SENSOR_ERROR_INVALID_CAPABILITY:
            return "Invalid capability";
        case AETHERIS_SENSOR_ERROR_DAO_POLICY_DENIED:
            return "DAO policy denied";
        case AETHERIS_SENSOR_ERROR_INVALID_SAMPLE_RATE:
            return "Invalid sample rate";
        case AETHERIS_SENSOR_ERROR_UNKNOWN:
        default:
            return "Unknown error";
    }
}

void aetheris_sensor_desc_default(
    aetheris_sensor_desc_t* desc,
    aetheris_sensor_kind_t kind,
    const char* name
) {
    if (!desc) {
        return;
    }
    
    strncpy(desc->name, name ? name : "Unknown Sensor", sizeof(desc->name) - 1);
    desc->name[sizeof(desc->name) - 1] = '\0';
    
    desc->kind = kind;
    
    // Set defaults based on sensor kind
    switch (kind) {
        case AETHERIS_SENSOR_ACCELEROMETER:
            strcpy(desc->location, "IMU");
            strcpy(desc->unit, "m/s²");
            desc->range_min = -20.0;
            desc->range_max = 20.0;
            desc->resolution = 0.01;
            desc->sample_rates[0] = 10;
            desc->sample_rates[1] = 50;
            desc->sample_rates[2] = 100;
            desc->sample_rates[3] = 200;
            desc->sample_rate_count = 4;
            break;
            
        case AETHERIS_SENSOR_GYROSCOPE:
            strcpy(desc->location, "IMU");
            strcpy(desc->unit, "deg/s");
            desc->range_min = -1000.0;
            desc->range_max = 1000.0;
            desc->resolution = 0.1;
            desc->sample_rates[0] = 10;
            desc->sample_rates[1] = 100;
            desc->sample_rates[2] = 1000;
            desc->sample_rate_count = 3;
            break;
            
        case AETHERIS_SENSOR_TEMPERATURE:
            strcpy(desc->location, "CPU");
            strcpy(desc->unit, "°C");
            desc->range_min = 0.0;
            desc->range_max = 100.0;
            desc->resolution = 0.1;
            desc->sample_rates[0] = 1;
            desc->sample_rates[1] = 10;
            desc->sample_rates[2] = 100;
            desc->sample_rate_count = 3;
            break;
            
        case AETHERIS_SENSOR_LIGHT:
            strcpy(desc->location, "Ambient");
            strcpy(desc->unit, "lux");
            desc->range_min = 0.0;
            desc->range_max = 1000.0;
            desc->resolution = 1.0;
            desc->sample_rates[0] = 1;
            desc->sample_rates[1] = 5;
            desc->sample_rates[2] = 10;
            desc->sample_rate_count = 3;
            break;
            
        default:
            strcpy(desc->location, "Unknown");
            strcpy(desc->unit, "units");
            desc->range_min = 0.0;
            desc->range_max = 100.0;
            desc->resolution = 1.0;
            desc->sample_rates[0] = 1;
            desc->sample_rates[1] = 10;
            desc->sample_rate_count = 2;
            break;
    }
}
