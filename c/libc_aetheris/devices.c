/**
 * @file devices.c
 * @brief C FFI implementation for Aetheris Device Runtime
 * 
 * This file provides the C implementation for device capture operations,
 * acting as a thin wrapper around the Rust device service.
 */

#include "devices.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

// Mock implementation for demonstration
// In a real implementation, this would call into the Rust service via FFI

static bool device_service_initialized = false;

aetheris_device_error_t aetheris_device_init(void) {
    if (device_service_initialized) {
        return AETHERIS_DEVICE_ERROR_INVALID_OPERATION;
    }
    
    // TODO: Initialize Rust device service
    device_service_initialized = true;
    return AETHERIS_DEVICE_SUCCESS;
}

aetheris_device_error_t aetheris_device_shutdown(void) {
    if (!device_service_initialized) {
        return AETHERIS_DEVICE_ERROR_INVALID_OPERATION;
    }
    
    // TODO: Shutdown Rust device service
    device_service_initialized = false;
    return AETHERIS_DEVICE_SUCCESS;
}

aetheris_device_error_t aetheris_camera_start(
    const char* session_id,
    const char* caps,
    const aetheris_capture_config_t* config,
    char* capture_id
) {
    if (!device_service_initialized) {
        return AETHERIS_DEVICE_ERROR_INVALID_OPERATION;
    }
    
    if (!session_id || !caps || !config || !capture_id) {
        return AETHERIS_DEVICE_ERROR_INVALID_PARAM;
    }
    
    // Mock implementation - generate a capture ID
    snprintf(capture_id, 64, "camera_capture_%ld", (long)time(NULL));
    
    // TODO: Call Rust device service
    // device_service_start_camera_capture(session_id, caps, config, capture_id);
    
    return AETHERIS_DEVICE_SUCCESS;
}

aetheris_device_error_t aetheris_camera_stop(
    const char* capture_id,
    aetheris_capture_stats_t* stats
) {
    if (!device_service_initialized) {
        return AETHERIS_DEVICE_ERROR_INVALID_OPERATION;
    }
    
    if (!capture_id || !stats) {
        return AETHERIS_DEVICE_ERROR_INVALID_PARAM;
    }
    
    // Mock implementation
    strncpy(stats->capture_id, capture_id, sizeof(stats->capture_id) - 1);
    stats->capture_id[sizeof(stats->capture_id) - 1] = '\0';
    
    stats->duration_ms = 5000; // 5 seconds
    stats->bytes_written = 1024 * 1024; // 1MB
    stats->chunks_created = 3;
    stats->last_chunk_timestamp = (uint64_t)time(NULL);
    
    snprintf(stats->snapshot_id, sizeof(stats->snapshot_id), "snapshot_%ld", (long)time(NULL));
    
    // TODO: Call Rust device service
    // device_service_stop_camera_capture(capture_id, stats);
    
    return AETHERIS_DEVICE_SUCCESS;
}

aetheris_device_error_t aetheris_microphone_start(
    const char* session_id,
    const char* caps,
    const aetheris_capture_config_t* config,
    char* capture_id
) {
    if (!device_service_initialized) {
        return AETHERIS_DEVICE_ERROR_INVALID_OPERATION;
    }
    
    if (!session_id || !caps || !config || !capture_id) {
        return AETHERIS_DEVICE_ERROR_INVALID_PARAM;
    }
    
    // Mock implementation - generate a capture ID
    snprintf(capture_id, 64, "mic_capture_%ld", (long)time(NULL));
    
    // TODO: Call Rust device service
    // device_service_start_microphone_capture(session_id, caps, config, capture_id);
    
    return AETHERIS_DEVICE_SUCCESS;
}

aetheris_device_error_t aetheris_microphone_stop(
    const char* capture_id,
    aetheris_capture_stats_t* stats
) {
    if (!device_service_initialized) {
        return AETHERIS_DEVICE_ERROR_INVALID_OPERATION;
    }
    
    if (!capture_id || !stats) {
        return AETHERIS_DEVICE_ERROR_INVALID_PARAM;
    }
    
    // Mock implementation
    strncpy(stats->capture_id, capture_id, sizeof(stats->capture_id) - 1);
    stats->capture_id[sizeof(stats->capture_id) - 1] = '\0';
    
    stats->duration_ms = 5000; // 5 seconds
    stats->bytes_written = 512 * 1024; // 512KB
    stats->chunks_created = 3;
    stats->last_chunk_timestamp = (uint64_t)time(NULL);
    
    snprintf(stats->snapshot_id, sizeof(stats->snapshot_id), "snapshot_%ld", (long)time(NULL));
    
    // TODO: Call Rust device service
    // device_service_stop_microphone_capture(capture_id, stats);
    
    return AETHERIS_DEVICE_SUCCESS;
}

aetheris_device_error_t aetheris_device_status(
    const char* capture_id,
    aetheris_capture_status_t* status
) {
    if (!device_service_initialized) {
        return AETHERIS_DEVICE_ERROR_INVALID_OPERATION;
    }
    
    if (!capture_id || !status) {
        return AETHERIS_DEVICE_ERROR_INVALID_PARAM;
    }
    
    // Mock implementation
    strncpy(status->capture_id, capture_id, sizeof(status->capture_id) - 1);
    status->capture_id[sizeof(status->capture_id) - 1] = '\0';
    
    status->running = true;
    status->bytes_written = 256 * 1024; // 256KB
    status->last_timestamp = (uint64_t)time(NULL);
    
    // TODO: Call Rust device service
    // device_service_get_capture_status(capture_id, status);
    
    return AETHERIS_DEVICE_SUCCESS;
}

aetheris_device_error_t aetheris_device_preview(
    const char* capture_id,
    const char* session_id,
    const char* caps,
    aetheris_preview_frame_t* frame
) {
    if (!device_service_initialized) {
        return AETHERIS_DEVICE_ERROR_INVALID_OPERATION;
    }
    
    if (!capture_id || !session_id || !caps || !frame) {
        return AETHERIS_DEVICE_ERROR_INVALID_PARAM;
    }
    
    // Mock implementation - generate a small preview frame
    frame->width = 160;
    frame->height = 90;
    frame->frame_size = frame->width * frame->height * 3; // RGB
    frame->timestamp = (uint64_t)time(NULL);
    
    // Allocate frame data
    frame->frame_data = malloc(frame->frame_size);
    if (!frame->frame_data) {
        return AETHERIS_DEVICE_ERROR_IO_ERROR;
    }
    
    // Fill with mock data
    memset(frame->frame_data, 128, frame->frame_size);
    
    // TODO: Call Rust device service
    // device_service_get_preview_frame(capture_id, session_id, caps, frame);
    
    return AETHERIS_DEVICE_SUCCESS;
}

void aetheris_preview_frame_free(aetheris_preview_frame_t* frame) {
    if (frame && frame->frame_data) {
        free(frame->frame_data);
        frame->frame_data = NULL;
        frame->frame_size = 0;
    }
}

const char* aetheris_device_error_message(aetheris_device_error_t error) {
    switch (error) {
        case AETHERIS_DEVICE_SUCCESS:
            return "Success";
        case AETHERIS_DEVICE_ERROR_INVALID_PARAM:
            return "Invalid parameter";
        case AETHERIS_DEVICE_ERROR_DEVICE_NOT_FOUND:
            return "Device not found";
        case AETHERIS_DEVICE_ERROR_CAPTURE_NOT_FOUND:
            return "Capture not found";
        case AETHERIS_DEVICE_ERROR_INVALID_CAPABILITY:
            return "Invalid capability";
        case AETHERIS_DEVICE_ERROR_DAO_POLICY_DENIED:
            return "DAO policy denied";
        case AETHERIS_DEVICE_ERROR_DEVICE_INIT_FAILED:
            return "Device initialization failed";
        case AETHERIS_DEVICE_ERROR_CAPTURE_START_FAILED:
            return "Capture start failed";
        case AETHERIS_DEVICE_ERROR_CAPTURE_STOP_FAILED:
            return "Capture stop failed";
        case AETHERIS_DEVICE_ERROR_INVALID_OPERATION:
            return "Invalid operation";
        case AETHERIS_DEVICE_ERROR_NGFS_ERROR:
            return "NGFS error";
        case AETHERIS_DEVICE_ERROR_ENCODING_ERROR:
            return "Encoding error";
        case AETHERIS_DEVICE_ERROR_IO_ERROR:
            return "I/O error";
        case AETHERIS_DEVICE_ERROR_SERIALIZATION_ERROR:
            return "Serialization error";
        case AETHERIS_DEVICE_ERROR_UNKNOWN:
        default:
            return "Unknown error";
    }
}

aetheris_device_error_t aetheris_device_set_dao_policy(
    const char* room_id,
    bool allow_camera,
    bool allow_microphone,
    bool allow_preview
) {
    if (!device_service_initialized) {
        return AETHERIS_DEVICE_ERROR_INVALID_OPERATION;
    }
    
    if (!room_id) {
        return AETHERIS_DEVICE_ERROR_INVALID_PARAM;
    }
    
    // TODO: Call Rust device service
    // device_service_set_dao_policy(room_id, allow_camera, allow_microphone, allow_preview);
    
    return AETHERIS_DEVICE_SUCCESS;
}

void aetheris_capture_config_default(aetheris_capture_config_t* config) {
    if (!config) {
        return;
    }
    
    // Camera defaults
    config->width = 640;
    config->height = 360;
    config->fps = 15;
    
    // Microphone defaults
    config->sample_rate = 44100;
    config->channels = 2;
    
    // Common defaults
    config->deterministic = false;
    config->max_chunk_duration_ms = 2000;
    config->max_chunk_frames = 30;
}
