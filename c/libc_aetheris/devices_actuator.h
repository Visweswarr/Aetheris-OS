/**
 * @file devices_actuator.h
 * @brief C FFI header for Aetheris Actuator Device Runtime
 * 
 * This header provides the C interface for actuator operations,
 * including PWM outputs, relays, motor control, LEDs, and
 * deterministic state management with NGFS snapshot integration.
 */

#ifndef AETHERIS_DEVICES_ACTUATOR_H
#define AETHERIS_DEVICES_ACTUATOR_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Actuator error codes
 */
typedef enum {
    AETHERIS_ACTUATOR_SUCCESS = 0,
    AETHERIS_ACTUATOR_ERROR_INVALID_PARAM = -1,
    AETHERIS_ACTUATOR_ERROR_ACTUATOR_NOT_CONFIGURED = -2,
    AETHERIS_ACTUATOR_ERROR_ACTUATOR_DISABLED = -3,
    AETHERIS_ACTUATOR_ERROR_INVALID_CAPABILITY = -4,
    AETHERIS_ACTUATOR_ERROR_DAO_POLICY_DENIED = -5,
    AETHERIS_ACTUATOR_ERROR_VALUE_OUT_OF_RANGE = -6,
    AETHERIS_ACTUATOR_ERROR_SAFETY_VIOLATION = -7,
    AETHERIS_ACTUATOR_ERROR_PATTERN_NOT_FOUND = -8,
    AETHERIS_ACTUATOR_ERROR_OPERATION_FAILED = -9,
    AETHERIS_ACTUATOR_ERROR_SERIALIZATION_ERROR = -10,
    AETHERIS_ACTUATOR_ERROR_DESERIALIZATION_ERROR = -11,
    AETHERIS_ACTUATOR_ERROR_UNKNOWN = -999
} aetheris_actuator_error_t;

/**
 * @brief Actuator types
 */
typedef enum {
    AETHERIS_ACTUATOR_TYPE_PWM = 0,
    AETHERIS_ACTUATOR_TYPE_RELAY = 1,
    AETHERIS_ACTUATOR_TYPE_MOTOR = 2,
    AETHERIS_ACTUATOR_TYPE_LED = 3,
    AETHERIS_ACTUATOR_TYPE_SERVO = 4,
    AETHERIS_ACTUATOR_TYPE_STEPPER = 5,
    AETHERIS_ACTUATOR_TYPE_SOLENOID = 6,
    AETHERIS_ACTUATOR_TYPE_VALVE = 7
} aetheris_actuator_type_t;

/**
 * @brief Actuator configuration
 */
typedef struct {
    char name[64];              ///< Actuator name
    aetheris_actuator_type_t type; ///< Actuator type
    uint32_t pin;               ///< Control pin
    float min_value;            ///< Minimum value
    float max_value;            ///< Maximum value
    float default_value;        ///< Default value
    uint32_t frequency;         ///< PWM frequency (Hz)
    uint32_t resolution;        ///< PWM resolution (bits)
    bool enable_safety_limits;  ///< Enable safety limits
    float safety_min;           ///< Safety minimum value
    float safety_max;           ///< Safety maximum value
    uint32_t ramp_time_ms;      ///< Ramp time in milliseconds
    bool enable_ramping;        ///< Enable value ramping
} aetheris_actuator_config_t;

/**
 * @brief Actuator state
 */
typedef struct {
    char name[64];              ///< Actuator name
    aetheris_actuator_type_t type; ///< Actuator type
    bool configured;            ///< Whether actuator is configured
    bool enabled;               ///< Whether actuator is enabled
    float current_value;        ///< Current output value
    float target_value;         ///< Target value
    uint64_t last_update;       ///< Last update timestamp
    uint64_t update_count;      ///< Total number of updates
    bool safety_enabled;        ///< Whether safety limits are enabled
    float safety_min;           ///< Current safety minimum
    float safety_max;           ///< Current safety maximum
    bool ramping_enabled;       ///< Whether ramping is enabled
    uint32_t ramp_time_ms;      ///< Current ramp time
    bool deterministic;         ///< Whether in deterministic mode
} aetheris_actuator_state_t;

/**
 * @brief Actuator operation result
 */
typedef struct {
    char operation_id[64];      ///< Unique operation identifier
    char actuator_name[64];     ///< Actuator name
    uint32_t operation;         ///< Operation type
    bool result;                ///< Operation result
    uint64_t timestamp;         ///< Operation timestamp
    bool deterministic;         ///< Whether operation was deterministic
} aetheris_actuator_operation_result_t;

/**
 * @brief Actuator operation types
 */
typedef enum {
    AETHERIS_ACTUATOR_OP_CONFIGURE = 0,
    AETHERIS_ACTUATOR_OP_ENABLE = 1,
    AETHERIS_ACTUATOR_OP_DISABLE = 2,
    AETHERIS_ACTUATOR_OP_SET_VALUE = 3,
    AETHERIS_ACTUATOR_OP_SET_PATTERN = 4,
    AETHERIS_ACTUATOR_OP_STOP_PATTERN = 5,
    AETHERIS_ACTUATOR_OP_SET_SAFETY_LIMITS = 6,
    AETHERIS_ACTUATOR_OP_ENABLE_RAMPING = 7,
    AETHERIS_ACTUATOR_OP_DISABLE_RAMPING = 8,
    AETHERIS_ACTUATOR_OP_SET_RAMP_TIME = 9,
    AETHERIS_ACTUATOR_OP_EMERGENCY_STOP = 10
} aetheris_actuator_operation_t;

/**
 * @brief Actuator pattern step
 */
typedef struct {
    float value;                ///< Step value
    uint32_t duration_ms;       ///< Step duration in milliseconds
} aetheris_actuator_pattern_step_t;

/**
 * @brief Actuator pattern
 */
typedef struct {
    char name[64];              ///< Pattern name
    uint32_t step_count;        ///< Number of steps
    aetheris_actuator_pattern_step_t* steps; ///< Pattern steps
    bool loop;                  ///< Whether to loop the pattern
    uint32_t loop_count;        ///< Number of loops (0 = infinite)
} aetheris_actuator_pattern_t;

/**
 * @brief Initialize actuator service
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_init(void);

/**
 * @brief Shutdown actuator service
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_shutdown(void);

/**
 * @brief Configure actuator
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param config Actuator configuration
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_configure(
    const char* session_id,
    const char* caps,
    const aetheris_actuator_config_t* config
);

/**
 * @brief Enable actuator
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param actuator_name Actuator name
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_enable(
    const char* session_id,
    const char* caps,
    const char* actuator_name
);

/**
 * @brief Disable actuator
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param actuator_name Actuator name
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_disable(
    const char* session_id,
    const char* caps,
    const char* actuator_name
);

/**
 * @brief Set actuator value
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param actuator_name Actuator name
 * @param value Value to set
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_set_value(
    const char* session_id,
    const char* caps,
    const char* actuator_name,
    float value
);

/**
 * @brief Get actuator value
 * @param actuator_name Actuator name
 * @param value Current value
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_get_value(
    const char* actuator_name,
    float* value
);

/**
 * @brief Set actuator pattern
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param actuator_name Actuator name
 * @param pattern Pattern to set
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_set_pattern(
    const char* session_id,
    const char* caps,
    const char* actuator_name,
    const aetheris_actuator_pattern_t* pattern
);

/**
 * @brief Stop actuator pattern
 * @param session_id Session identifier
 * @param caps Capability tokens
 * @param actuator_name Actuator name
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_stop_pattern(
    const char* session_id,
    const char* caps,
    const char* actuator_name
);

/**
 * @brief Get actuator state
 * @param actuator_name Actuator name
 * @param state Actuator state
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_get_state(
    const char* actuator_name,
    aetheris_actuator_state_t* state
);

/**
 * @brief Get all configured actuators
 * @param states Array of actuator states
 * @param count Number of actuators
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_get_configured_actuators(
    aetheris_actuator_state_t** states,
    size_t* count
);

/**
 * @brief Free actuator states array
 * @param states Array of actuator states
 */
void aetheris_actuator_free_actuator_states(aetheris_actuator_state_t* states);

/**
 * @brief Get actuator operation history
 * @param limit Maximum number of operations to return
 * @param operations Array of operation results
 * @param count Number of operations
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_get_operation_history(
    size_t limit,
    aetheris_actuator_operation_result_t** operations,
    size_t* count
);

/**
 * @brief Free actuator operation history array
 * @param operations Array of operation results
 */
void aetheris_actuator_free_operation_history(aetheris_actuator_operation_result_t* operations);

/**
 * @brief Enable deterministic mode
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_enable_deterministic_mode(void);

/**
 * @brief Disable deterministic mode
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_disable_deterministic_mode(void);

/**
 * @brief Advance tick counter
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_advance_tick(void);

/**
 * @brief Get tick counter
 * @param tick_count Tick counter value
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_get_tick_counter(uint64_t* tick_count);

/**
 * @brief Create NGFS snapshot
 * @param snapshot_data Snapshot data
 * @param snapshot_size Snapshot size
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_create_snapshot(
    uint8_t** snapshot_data,
    size_t* snapshot_size
);

/**
 * @brief Restore NGFS snapshot
 * @param snapshot_data Snapshot data
 * @param snapshot_size Snapshot size
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_restore_snapshot(
    const uint8_t* snapshot_data,
    size_t snapshot_size
);

/**
 * @brief Free snapshot data
 * @param snapshot_data Snapshot data
 */
void aetheris_actuator_free_snapshot(uint8_t* snapshot_data);

/**
 * @brief Get error message
 * @param error Error code
 * @return Error message
 */
const char* aetheris_actuator_error_message(aetheris_actuator_error_t error);

/**
 * @brief Set default actuator configuration
 * @param config Configuration to initialize
 */
void aetheris_actuator_config_default(aetheris_actuator_config_t* config);

/**
 * @brief Set DAO policy for actuator operations
 * @param room_id Room identifier
 * @param allow_configure Allow actuator configuration
 * @param allow_control Allow actuator control
 * @return Error code
 */
aetheris_actuator_error_t aetheris_actuator_set_dao_policy(
    const char* room_id,
    bool allow_configure,
    bool allow_control
);

#ifdef __cplusplus
}
#endif

#endif // AETHERIS_DEVICES_ACTUATOR_H
