/**
 * @file devices.h
 * @brief C FFI interface for Aetheris Device Runtime
 * 
 * This header provides a thin C interface for device capture operations,
 * including camera and microphone capture with capability gating and
 * deterministic recording to NGFS snapshots.
 */

#ifndef AETHERIS_DEVICES_H
#define AETHERIS_DEVICES_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Device capture configuration
 */
typedef struct {
    // Camera configuration
    uint32_t width;
    uint32_t height;
    uint32_t fps;
    
    // Microphone configuration
    uint32_t sample_rate;
    uint16_t channels;
    
    // Common configuration
    bool deterministic;
    uint32_t max_chunk_duration_ms;
    uint32_t max_chunk_frames;
} aetheris_capture_config_t;

/**
 * @brief Capture statistics
 */
typedef struct {
    char capture_id[64];
    uint64_t duration_ms;
    uint64_t bytes_written;
    uint32_t chunks_created;
    uint64_t last_chunk_timestamp;
    char snapshot_id[64];
} aetheris_capture_stats_t;

/**
 * @brief Capture status
 */
typedef struct {
    char capture_id[64];
    bool running;
    uint64_t bytes_written;
    uint64_t last_timestamp;
} aetheris_capture_status_t;

/**
 * @brief Preview frame data
 */
typedef struct {
    uint8_t* frame_data;
    size_t frame_size;
    uint32_t width;
    uint32_t height;
    uint64_t timestamp;
} aetheris_preview_frame_t;

/**
 * @brief Error codes
 */
typedef enum {
    AETHERIS_DEVICE_SUCCESS = 0,
    AETHERIS_DEVICE_ERROR_INVALID_PARAM = -1,
    AETHERIS_DEVICE_ERROR_DEVICE_NOT_FOUND = -2,
    AETHERIS_DEVICE_ERROR_CAPTURE_NOT_FOUND = -3,
    AETHERIS_DEVICE_ERROR_INVALID_CAPABILITY = -4,
    AETHERIS_DEVICE_ERROR_DAO_POLICY_DENIED = -5,
    AETHERIS_DEVICE_ERROR_DEVICE_INIT_FAILED = -6,
    AETHERIS_DEVICE_ERROR_CAPTURE_START_FAILED = -7,
    AETHERIS_DEVICE_ERROR_CAPTURE_STOP_FAILED = -8,
    AETHERIS_DEVICE_ERROR_INVALID_OPERATION = -9,
    AETHERIS_DEVICE_ERROR_NGFS_ERROR = -10,
    AETHERIS_DEVICE_ERROR_ENCODING_ERROR = -11,
    AETHERIS_DEVICE_ERROR_IO_ERROR = -12,
    AETHERIS_DEVICE_ERROR_SERIALIZATION_ERROR = -13,
    AETHERIS_DEVICE_ERROR_UNKNOWN = -999
} aetheris_device_error_t;

/**
 * @brief Initialize the device service
 * 
 * @return AETHERIS_DEVICE_SUCCESS on success, error code on failure
 */
aetheris_device_error_t aetheris_device_init(void);

/**
 * @brief Shutdown the device service
 * 
 * @return AETHERIS_DEVICE_SUCCESS on success, error code on failure
 */
aetheris_device_error_t aetheris_device_shutdown(void);

/**
 * @brief Start camera capture
 * 
 * @param session_id Session identifier
 * @param caps Capability string (comma-separated)
 * @param config Capture configuration
 * @param capture_id Output capture identifier (must be at least 64 bytes)
 * @return AETHERIS_DEVICE_SUCCESS on success, error code on failure
 */
aetheris_device_error_t aetheris_camera_start(
    const char* session_id,
    const char* caps,
    const aetheris_capture_config_t* config,
    char* capture_id
);

/**
 * @brief Stop camera capture
 * 
 * @param capture_id Capture identifier
 * @param stats Output capture statistics
 * @return AETHERIS_DEVICE_SUCCESS on success, error code on failure
 */
aetheris_device_error_t aetheris_camera_stop(
    const char* capture_id,
    aetheris_capture_stats_t* stats
);

/**
 * @brief Start microphone capture
 * 
 * @param session_id Session identifier
 * @param caps Capability string (comma-separated)
 * @param config Capture configuration
 * @param capture_id Output capture identifier (must be at least 64 bytes)
 * @return AETHERIS_DEVICE_SUCCESS on success, error code on failure
 */
aetheris_device_error_t aetheris_microphone_start(
    const char* session_id,
    const char* caps,
    const aetheris_capture_config_t* config,
    char* capture_id
);

/**
 * @brief Stop microphone capture
 * 
 * @param capture_id Capture identifier
 * @param stats Output capture statistics
 * @return AETHERIS_DEVICE_SUCCESS on success, error code on failure
 */
aetheris_device_error_t aetheris_microphone_stop(
    const char* capture_id,
    aetheris_capture_stats_t* stats
);

/**
 * @brief Get capture status
 * 
 * @param capture_id Capture identifier
 * @param status Output capture status
 * @return AETHERIS_DEVICE_SUCCESS on success, error code on failure
 */
aetheris_device_error_t aetheris_device_status(
    const char* capture_id,
    aetheris_capture_status_t* status
);

/**
 * @brief Get preview frame (camera only)
 * 
 * @param capture_id Capture identifier
 * @param session_id Session identifier
 * @param caps Capability string (comma-separated)
 * @param frame Output preview frame (caller must free frame_data)
 * @return AETHERIS_DEVICE_SUCCESS on success, error code on failure
 */
aetheris_device_error_t aetheris_device_preview(
    const char* capture_id,
    const char* session_id,
    const char* caps,
    aetheris_preview_frame_t* frame
);

/**
 * @brief Free preview frame data
 * 
 * @param frame Preview frame to free
 */
void aetheris_preview_frame_free(aetheris_preview_frame_t* frame);

/**
 * @brief Get error message for error code
 * 
 * @param error Error code
 * @return Error message string
 */
const char* aetheris_device_error_message(aetheris_device_error_t error);

/**
 * @brief Set DAO policy for a room
 * 
 * @param room_id Room identifier
 * @param allow_camera Allow camera capture
 * @param allow_microphone Allow microphone capture
 * @param allow_preview Allow preview access
 * @return AETHERIS_DEVICE_SUCCESS on success, error code on failure
 */
aetheris_device_error_t aetheris_device_set_dao_policy(
    const char* room_id,
    bool allow_camera,
    bool allow_microphone,
    bool allow_preview
);

/**
 * @brief Get default capture configuration
 * 
 * @param config Output configuration
 */
void aetheris_capture_config_default(aetheris_capture_config_t* config);

#ifdef __cplusplus
}
#endif

#endif // AETHERIS_DEVICES_H
