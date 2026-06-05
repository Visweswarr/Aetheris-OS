/**
 * @file devices_gpio.h
 * @brief C FFI interface for Aetheris GPIO Device Runtime
 * 
 * This header provides a thin C interface for GPIO operations,
 * including pin configuration, digital I/O, and deterministic
 * state management with NGFS snapshot integration.
 */

#ifndef AETHERIS_DEVICES_GPIO_H
#define AETHERIS_DEVICES_GPIO_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief GPIO pin modes
 */
typedef enum {
    AETHERIS_GPIO_MODE_INPUT = 0,
    AETHERIS_GPIO_MODE_OUTPUT = 1,
    AETHERIS_GPIO_MODE_INPUT_PULLUP = 2,
    AETHERIS_GPIO_MODE_INPUT_PULLDOWN = 3,
    AETHERIS_GPIO_MODE_OUTPUT_OPENDRAIN = 4,
    AETHERIS_GPIO_MODE_OUTPUT_PUSHPULL = 5
} aetheris_gpio_mode_t;

/**
 * @brief GPIO pull resistor configuration
 */
typedef enum {
    AETHERIS_GPIO_PULL_NONE = 0,
    AETHERIS_GPIO_PULL_UP = 1,
    AETHERIS_GPIO_PULL_DOWN = 2
} aetheris_gpio_pull_t;

/**
 * @brief GPIO pin configuration
 */
typedef struct {
    uint32_t pin;
    aetheris_gpio_mode_t mode;
    aetheris_gpio_pull_t pull;
    bool initial_value;
    uint32_t debounce_ms;
} aetheris_gpio_config_t;

/**
 * @brief GPIO pin state
 */
typedef struct {
    uint32_t pin;
    aetheris_gpio_mode_t mode;
    aetheris_gpio_pull_t pull;
    bool value;
    uint64_t last_change;
    uint64_t change_count;
} aetheris_gpio_state_t;

/**
 * @brief GPIO operation types
 */
typedef enum {
    AETHERIS_GPIO_OP_READ = 0,
    AETHERIS_GPIO_OP_WRITE = 1,
    AETHERIS_GPIO_OP_CONFIGURE = 2,
    AETHERIS_GPIO_OP_TOGGLE = 3
} aetheris_gpio_operation_t;

/**
 * @brief GPIO operation result
 */
typedef struct {
    char operation_id[64];
    uint32_t pin;
    aetheris_gpio_operation_t operation;
    bool result;
    uint64_t timestamp;
    bool deterministic;
} aetheris_gpio_operation_result_t;

/**
 * @brief GPIO error codes
 */
typedef enum {
    AETHERIS_GPIO_SUCCESS = 0,
    AETHERIS_GPIO_ERROR_INVALID_PARAM = -1,
    AETHERIS_GPIO_ERROR_PIN_NOT_CONFIGURED = -2,
    AETHERIS_GPIO_ERROR_INVALID_PIN_MODE = -3,
    AETHERIS_GPIO_ERROR_INVALID_CAPABILITY = -4,
    AETHERIS_GPIO_ERROR_DAO_POLICY_DENIED = -5,
    AETHERIS_GPIO_ERROR_OPERATION_FAILED = -6,
    AETHERIS_GPIO_ERROR_SERIALIZATION_ERROR = -7,
    AETHERIS_GPIO_ERROR_DESERIALIZATION_ERROR = -8,
    AETHERIS_GPIO_ERROR_UNKNOWN = -999
} aetheris_gpio_error_t;

/**
 * @brief Initialize GPIO manager
 * 
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_init(void);

/**
 * @brief Shutdown GPIO manager
 * 
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_shutdown(void);

/**
 * @brief Configure a GPIO pin
 * 
 * @param session_id Session identifier
 * @param caps Capability string (comma-separated)
 * @param config Pin configuration
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_configure_pin(
    const char* session_id,
    const char* caps,
    const aetheris_gpio_config_t* config
);

/**
 * @brief Read a GPIO pin value
 * 
 * @param session_id Session identifier
 * @param caps Capability string (comma-separated)
 * @param pin Pin number
 * @param value Output pin value
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_read_pin(
    const char* session_id,
    const char* caps,
    uint32_t pin,
    bool* value
);

/**
 * @brief Write a GPIO pin value
 * 
 * @param session_id Session identifier
 * @param caps Capability string (comma-separated)
 * @param pin Pin number
 * @param value Pin value to write
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_write_pin(
    const char* session_id,
    const char* caps,
    uint32_t pin,
    bool value
);

/**
 * @brief Toggle a GPIO pin value
 * 
 * @param session_id Session identifier
 * @param caps Capability string (comma-separated)
 * @param pin Pin number
 * @param new_value Output new pin value after toggle
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_toggle_pin(
    const char* session_id,
    const char* caps,
    uint32_t pin,
    bool* new_value
);

/**
 * @brief Get GPIO pin state
 * 
 * @param pin Pin number
 * @param state Output pin state
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_get_pin_state(
    uint32_t pin,
    aetheris_gpio_state_t* state
);

/**
 * @brief Get all configured GPIO pins
 * 
 * @param states Output array of pin states (caller must free)
 * @param count Output number of configured pins
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_get_configured_pins(
    aetheris_gpio_state_t** states,
    size_t* count
);

/**
 * @brief Free GPIO pin states array
 * 
 * @param states Pin states array to free
 */
void aetheris_gpio_free_pin_states(aetheris_gpio_state_t* states);

/**
 * @brief Get GPIO operation history
 * 
 * @param limit Maximum number of operations to return (0 for all)
 * @param operations Output array of operations (caller must free)
 * @param count Output number of operations
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_get_operation_history(
    size_t limit,
    aetheris_gpio_operation_result_t** operations,
    size_t* count
);

/**
 * @brief Free GPIO operation history array
 * 
 * @param operations Operations array to free
 */
void aetheris_gpio_free_operation_history(aetheris_gpio_operation_result_t* operations);

/**
 * @brief Enable deterministic mode
 * 
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_enable_deterministic_mode(void);

/**
 * @brief Disable deterministic mode
 * 
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_disable_deterministic_mode(void);

/**
 * @brief Advance tick counter (for deterministic operations)
 * 
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_advance_tick(void);

/**
 * @brief Get current tick counter
 * 
 * @param tick_count Output tick counter value
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_get_tick_counter(uint64_t* tick_count);

/**
 * @brief Create NGFS snapshot of GPIO state
 * 
 * @param snapshot_data Output snapshot data (caller must free)
 * @param snapshot_size Output snapshot size
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_create_snapshot(
    uint8_t** snapshot_data,
    size_t* snapshot_size
);

/**
 * @brief Restore GPIO state from NGFS snapshot
 * 
 * @param snapshot_data Snapshot data
 * @param snapshot_size Snapshot size
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_restore_snapshot(
    const uint8_t* snapshot_data,
    size_t snapshot_size
);

/**
 * @brief Free snapshot data
 * 
 * @param snapshot_data Snapshot data to free
 */
void aetheris_gpio_free_snapshot(uint8_t* snapshot_data);

/**
 * @brief Get error message for error code
 * 
 * @param error Error code
 * @return Error message string
 */
const char* aetheris_gpio_error_message(aetheris_gpio_error_t error);

/**
 * @brief Get default GPIO configuration
 * 
 * @param config Output configuration
 */
void aetheris_gpio_config_default(aetheris_gpio_config_t* config);

/**
 * @brief Set DAO policy for GPIO operations
 * 
 * @param room_id Room identifier
 * @param allow_configure Allow pin configuration
 * @param allow_read Allow pin reading
 * @param allow_write Allow pin writing
 * @return AETHERIS_GPIO_SUCCESS on success, error code on failure
 */
aetheris_gpio_error_t aetheris_gpio_set_dao_policy(
    const char* room_id,
    bool allow_configure,
    bool allow_read,
    bool allow_write
);

#ifdef __cplusplus
}
#endif

#endif // AETHERIS_DEVICES_GPIO_H
