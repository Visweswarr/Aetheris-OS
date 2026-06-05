/**
 * @file devices_sensor.h
 * @brief C FFI interface for Aetheris Sensor Device Runtime
 * 
 * This header provides a thin C interface for sensor operations,
 * including sensor registration, sampling, preview streams, and
 * deterministic snapshots with capability gating.
 */

#ifndef AETHERIS_DEVICES_SENSOR_H
#define AETHERIS_DEVICES_SENSOR_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Sensor kind enumeration
 */
typedef enum {
    AETHERIS_SENSOR_ACCELEROMETER = 0,
    AETHERIS_SENSOR_GYROSCOPE = 1,
    AETHERIS_SENSOR_MAGNETOMETER = 2,
    AETHERIS_SENSOR_TEMPERATURE = 3,
    AETHERIS_SENSOR_HUMIDITY = 4,
    AETHERIS_SENSOR_PRESSURE = 5,
    AETHERIS_SENSOR_LIGHT = 6,
    AETHERIS_SENSOR_PROXIMITY = 7,
    AETHERIS_SENSOR_HEART_RATE = 8,
    AETHERIS_SENSOR_CUSTOM = 9
} aetheris_sensor_kind_t;

/**
 * @brief Sensor value type
 */
typedef enum {
    AETHERIS_SENSOR_VALUE_SCALAR = 0,
    AETHERIS_SENSOR_VALUE_VECTOR2 = 1,
    AETHERIS_SENSOR_VALUE_VECTOR3 = 2
} aetheris_sensor_value_type_t;

/**
 * @brief Sensor value union
 */
typedef union {
    double scalar;
    struct {
        double x;
        double y;
    } vector2;
    struct {
        double x;
        double y;
        double z;
    } vector3;
} aetheris_sensor_value_t;

/**
 * @brief Sensor description for registration
 */
typedef struct {
    char name[64];
    aetheris_sensor_kind_t kind;
    char location[32];
    char unit[16];
    double range_min;
    double range_max;
    double resolution;
    uint32_t sample_rates[8];  // Supported sample rates
    uint8_t sample_rate_count;
} aetheris_sensor_desc_t;

/**
 * @brief Sensor information
 */
typedef struct {
    char sensor_id[64];
    char name[64];
    aetheris_sensor_kind_t kind;
    char location[32];
    char unit[16];
    double range_min;
    double range_max;
    double resolution;
    uint32_t sample_rates[8];
    uint8_t sample_rate_count;
    uint64_t registered_at_timestamp;
    bool active_sampling;
} aetheris_sensor_info_t;

/**
 * @brief Sensor sample data
 */
typedef struct {
    char sensor_id[64];
    uint64_t timestamp;
    aetheris_sensor_value_type_t value_type;
    aetheris_sensor_value_t value;
    uint64_t sequence_number;
} aetheris_sensor_sample_t;

/**
 * @brief Error codes for sensor operations
 */
typedef enum {
    AETHERIS_SENSOR_SUCCESS = 0,
    AETHERIS_SENSOR_ERROR_INVALID_PARAM = -1,
    AETHERIS_SENSOR_ERROR_DEVICE_NOT_FOUND = -2,
    AETHERIS_SENSOR_ERROR_REGISTRATION_FAILED = -3,
    AETHERIS_SENSOR_ERROR_SAMPLING_FAILED = -4,
    AETHERIS_SENSOR_ERROR_PREVIEW_FAILED = -5,
    AETHERIS_SENSOR_ERROR_SNAPSHOT_FAILED = -6,
    AETHERIS_SENSOR_ERROR_INVALID_CAPABILITY = -7,
    AETHERIS_SENSOR_ERROR_DAO_POLICY_DENIED = -8,
    AETHERIS_SENSOR_ERROR_INVALID_SAMPLE_RATE = -9,
    AETHERIS_SENSOR_ERROR_UNKNOWN = -999
} aetheris_sensor_error_t;

/**
 * @brief Initialize sensor runtime
 * 
 * @return AETHERIS_SENSOR_SUCCESS on success, error code on failure
 */
aetheris_sensor_error_t aetheris_sensor_init(void);

/**
 * @brief Shutdown sensor runtime
 * 
 * @return AETHERIS_SENSOR_SUCCESS on success, error code on failure
 */
aetheris_sensor_error_t aetheris_sensor_shutdown(void);

/**
 * @brief Register a sensor
 * 
 * @param desc Sensor description
 * @param sensor_id Output sensor ID (must be at least 64 bytes)
 * @return AETHERIS_SENSOR_SUCCESS on success, error code on failure
 */
aetheris_sensor_error_t aetheris_sensor_register(
    const aetheris_sensor_desc_t* desc,
    char* sensor_id
);

/**
 * @brief List all registered sensors
 * 
 * @param sensors Output array of sensor info
 * @param max_sensors Maximum number of sensors to return
 * @param sensor_count Output number of sensors found
 * @return AETHERIS_SENSOR_SUCCESS on success, error code on failure
 */
aetheris_sensor_error_t aetheris_sensor_list(
    aetheris_sensor_info_t* sensors,
    size_t max_sensors,
    size_t* sensor_count
);

/**
 * @brief Start sensor sampling
 * 
 * @param session_id Session identifier
 * @param caps Capability string (comma-separated)
 * @param sensor_id Sensor ID
 * @param hz Sample rate in Hz
 * @param seed Deterministic seed (0 for non-deterministic)
 * @param handle Output sampling handle (must be at least 64 bytes)
 * @return AETHERIS_SENSOR_SUCCESS on success, error code on failure
 */
aetheris_sensor_error_t aetheris_sensor_start_sampling(
    const char* session_id,
    const char* caps,
    const char* sensor_id,
    uint32_t hz,
    uint64_t seed,
    char* handle
);

/**
 * @brief Stop sensor sampling
 * 
 * @param handle Sampling handle
 * @return AETHERIS_SENSOR_SUCCESS on success, error code on failure
 */
aetheris_sensor_error_t aetheris_sensor_stop_sampling(const char* handle);

/**
 * @brief Start sensor preview stream
 * 
 * @param sensor_id Sensor ID
 * @param hz_max Maximum preview rate in Hz
 * @param stream_handle Output stream handle (must be at least 64 bytes)
 * @return AETHERIS_SENSOR_SUCCESS on success, error code on failure
 */
aetheris_sensor_error_t aetheris_sensor_preview(
    const char* sensor_id,
    uint32_t hz_max,
    char* stream_handle
);

/**
 * @brief Create sensor snapshot
 * 
 * @param sensor_id Sensor ID
 * @param out_ngfs_path Output NGFS path
 * @param snapshot_id Output snapshot ID (must be at least 64 bytes)
 * @return AETHERIS_SENSOR_SUCCESS on success, error code on failure
 */
aetheris_sensor_error_t aetheris_sensor_snapshot(
    const char* sensor_id,
    const char* out_ngfs_path,
    char* snapshot_id
);

/**
 * @brief Get sensor error message
 * 
 * @param error Error code
 * @return Error message string
 */
const char* aetheris_sensor_error_message(aetheris_sensor_error_t error);

/**
 * @brief Set default sensor description
 * 
 * @param desc Output description
 * @param kind Sensor kind
 * @param name Sensor name
 */
void aetheris_sensor_desc_default(
    aetheris_sensor_desc_t* desc,
    aetheris_sensor_kind_t kind,
    const char* name
);

#ifdef __cplusplus
}
#endif

#endif // AETHERIS_DEVICES_SENSOR_H
