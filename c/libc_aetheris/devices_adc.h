/**
 * @file devices_adc.h
 * @brief C FFI header for Aetheris ADC Device Runtime
 * 
 * This header provides the C interface for ADC operations,
 * including channel configuration, analog sampling, and
 * deterministic state management with NGFS snapshot integration.
 */

#ifndef AETHERIS_DEVICES_ADC_H
#define AETHERIS_DEVICES_ADC_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief ADC error codes
 */
typedef enum {
    AETHERIS_ADC_SUCCESS = 0,
    AETHERIS_ADC_ERROR_INVALID_PARAM = -1,
    AETHERIS_ADC_ERROR_CHANNEL_NOT_CONFIGURED = -2,
    AETHERIS_ADC_ERROR_CHANNEL_DISABLED = -3,
    AETHERIS_ADC_ERROR_INVALID_CAPABILITY = -4,
    AETHERIS_ADC_ERROR_DAO_POLICY_DENIED = -5,
    AETHERIS_ADC_ERROR_SAMPLING_FAILED = -6,
    AETHERIS_ADC_ERROR_SERIALIZATION_ERROR = -7,
    AETHERIS_ADC_ERROR_DESERIALIZATION_ERROR = -8,
    AETHERIS_ADC_ERROR_UNKNOWN = -999
} aetheris_adc_error_t;

/**
 * @brief ADC channel configuration
 */
typedef struct {
    uint32_t channel;           ///< ADC channel number
    uint32_t sample_rate;       ///< Sample rate in Hz
    uint32_t resolution;        ///< ADC resolution in bits
    float reference_voltage;    ///< Reference voltage in volts
    bool enable_calibration;    ///< Enable calibration
    float calibration_offset;   ///< Calibration offset
    float calibration_scale;    ///< Calibration scale factor
    uint32_t oversampling;      ///< Oversampling factor
    bool enable_filtering;      ///< Enable digital filtering
    float filter_cutoff;        ///< Filter cutoff frequency in Hz
} aetheris_adc_config_t;

/**
 * @brief ADC channel state
 */
typedef struct {
    uint32_t channel;           ///< Channel number
    bool configured;            ///< Whether channel is configured
    bool enabled;               ///< Whether channel is enabled
    uint32_t sample_rate;       ///< Current sample rate
    uint32_t resolution;        ///< Current resolution
    float reference_voltage;    ///< Current reference voltage
    float last_sample;          ///< Last sampled value
    uint64_t last_sample_time;  ///< Timestamp of last sample
    uint64_t sample_count;      ///< Total number of samples
    float min_value;            ///< Minimum value seen
    float max_value;            ///< Maximum value seen
    float average_value;        ///< Running average
    bool calibration_enabled;   ///< Whether calibration is enabled
    float calibration_offset;   ///< Current calibration offset
    float calibration_scale;    ///< Current calibration scale
} aetheris_adc_state_t;

/**
 * @brief ADC sample result
 */
typedef struct {
    uint32_t channel;           ///< Channel number
    float value;                ///< Sampled value (volts)
    float raw_value;            ///< Raw ADC value
    uint64_t timestamp;         ///< Sample timestamp
    bool deterministic;         ///< Whether sample was deterministic
    uint64_t tick_count;        ///< Tick count when sampled
} aetheris_adc_sample_t;

/**
 * @brief ADC operation result
 */
typedef struct {
    char operation_id[64];      ///< Unique operation identifier
    uint32_t channel;           ///< Channel number
    uint32_t operation;         ///< Operation type
    bool result;                ///< Operation result
    uint64_t timestamp;         ///< Operation timestamp
    bool deterministic;         ///< Whether operation was deterministic
} aetheris_adc_operation_result_t;

/**
 * @brief ADC operation types
 */
typedef enum {
    AETHERIS_ADC_OP_CONFIGURE = 0,
    AETHERIS_ADC_OP_ENABLE = 1,
    AETHERIS_ADC_OP_DISABLE = 2,
    AETHERIS_ADC_OP_SAMPLE = 3,
    AETHERIS_ADC_OP_CALIBRATE = 4,
    AETHERIS_ADC_OP_SET_REFERENCE = 5,
    AETHERIS_ADC_OP_SET_SAMPLE_RATE = 6,
    AETHERIS_ADC_OP_SET_RESOLUTION = 7,
    AETHERIS_ADC_OP_SET_OVERSAMPLING = 8,
    AETHERIS_ADC_OP_ENABLE_FILTER = 9,
    AETHERIS_ADC_OP_DISABLE_FILTER = 10
} aetheris_adc_operation_t;

/**
 * @brief Initialize ADC service
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_init(void);

/**
 * @brief Shutdown ADC service
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_shutdown(void);

/**
 * @brief Configure ADC channel
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param config Channel configuration
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_configure_channel(
    const char* session_id,
    const char* caps,
    const aetheris_adc_config_t* config
);

/**
 * @brief Enable ADC channel
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param channel Channel number
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_enable_channel(
    const char* session_id,
    const char* caps,
    uint32_t channel
);

/**
 * @brief Disable ADC channel
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param channel Channel number
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_disable_channel(
    const char* session_id,
    const char* caps,
    uint32_t channel
);

/**
 * @brief Sample ADC channel
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param channel Channel number
 * @param sample Sample result
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_sample_channel(
    const char* session_id,
    const char* caps,
    uint32_t channel,
    aetheris_adc_sample_t* sample
);

/**
 * @brief Sample multiple ADC channels
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param channels Array of channel numbers
 * @param channel_count Number of channels
 * @param samples Array of sample results
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_sample_channels(
    const char* session_id,
    const char* caps,
    const uint32_t* channels,
    size_t channel_count,
    aetheris_adc_sample_t* samples
);

/**
 * @brief Get ADC channel state
 * @param channel Channel number
 * @param state Channel state
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_get_channel_state(
    uint32_t channel,
    aetheris_adc_state_t* state
);

/**
 * @brief Get all configured ADC channels
 * @param states Array of channel states
 * @param count Number of channels
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_get_configured_channels(
    aetheris_adc_state_t** states,
    size_t* count
);

/**
 * @brief Free ADC channel states array
 * @param states Array of channel states
 */
void aetheris_adc_free_channel_states(aetheris_adc_state_t* states);

/**
 * @brief Get ADC operation history
 * @param limit Maximum number of operations to return
 * @param operations Array of operation results
 * @param count Number of operations
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_get_operation_history(
    size_t limit,
    aetheris_adc_operation_result_t** operations,
    size_t* count
);

/**
 * @brief Free ADC operation history array
 * @param operations Array of operation results
 */
void aetheris_adc_free_operation_history(aetheris_adc_operation_result_t* operations);

/**
 * @brief Enable deterministic mode
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_enable_deterministic_mode(void);

/**
 * @brief Disable deterministic mode
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_disable_deterministic_mode(void);

/**
 * @brief Advance tick counter
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_advance_tick(void);

/**
 * @brief Get tick counter
 * @param tick_count Tick counter value
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_get_tick_counter(uint64_t* tick_count);

/**
 * @brief Create NGFS snapshot
 * @param snapshot_data Snapshot data
 * @param snapshot_size Snapshot size
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_create_snapshot(
    uint8_t** snapshot_data,
    size_t* snapshot_size
);

/**
 * @brief Restore NGFS snapshot
 * @param snapshot_data Snapshot data
 * @param snapshot_size Snapshot size
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_restore_snapshot(
    const uint8_t* snapshot_data,
    size_t snapshot_size
);

/**
 * @brief Free snapshot data
 * @param snapshot_data Snapshot data
 */
void aetheris_adc_free_snapshot(uint8_t* snapshot_data);

/**
 * @brief Get error message
 * @param error Error code
 * @return Error message
 */
const char* aetheris_adc_error_message(aetheris_adc_error_t error);

/**
 * @brief Set default ADC configuration
 * @param config Configuration to initialize
 */
void aetheris_adc_config_default(aetheris_adc_config_t* config);

/**
 * @brief Set DAO policy for ADC operations
 * @param room_id Room identifier
 * @param allow_configure Allow channel configuration
 * @param allow_sample Allow channel sampling
 * @return Error code
 */
aetheris_adc_error_t aetheris_adc_set_dao_policy(
    const char* room_id,
    bool allow_configure,
    bool allow_sample
);

#ifdef __cplusplus
}
#endif

#endif // AETHERIS_DEVICES_ADC_H
