/**
 * @file devices_list.h
 * @brief C FFI interface for Aetheris Device Listing and Provider Management
 * 
 * This header provides a thin C interface for device discovery, listing,
 * and provider management operations.
 */

#ifndef AETHERIS_DEVICES_LIST_H
#define AETHERIS_DEVICES_LIST_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Device types
 */
typedef enum {
    AETHERIS_DEVICE_TYPE_CAMERA = 0,
    AETHERIS_DEVICE_TYPE_MICROPHONE = 1,
    AETHERIS_DEVICE_TYPE_GPIO = 2,
    AETHERIS_DEVICE_TYPE_ADC = 3,
    AETHERIS_DEVICE_TYPE_ACTUATOR = 4
} aetheris_device_type_t;

/**
 * @brief Provider types
 */
typedef enum {
    AETHERIS_PROVIDER_DETERMINISTIC = 0,
    AETHERIS_PROVIDER_LINUX = 1,
    AETHERIS_PROVIDER_CUSTOM = 2
} aetheris_provider_type_t;

/**
 * @brief Device information
 */
typedef struct {
    char device_id[128];
    aetheris_device_type_t device_type;
    char device_path[256];
    char capabilities[512];  // Comma-separated list
    aetheris_provider_type_t provider;
    bool is_available;
    char metadata[1024];     // JSON-like metadata
} aetheris_device_info_t;

/**
 * @brief Provider status information
 */
typedef struct {
    aetheris_provider_type_t provider_id;
    bool is_available;
    size_t device_count;
    char last_error[256];
    char capabilities[512];  // Comma-separated list
} aetheris_provider_status_t;

/**
 * @brief Device listing error codes
 */
typedef enum {
    AETHERIS_DEVICE_LIST_SUCCESS = 0,
    AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM = -1,
    AETHERIS_DEVICE_LIST_ERROR_DEVICE_NOT_FOUND = -2,
    AETHERIS_DEVICE_LIST_ERROR_PROVIDER_NOT_FOUND = -3,
    AETHERIS_DEVICE_LIST_ERROR_DISCOVERY_FAILED = -4,
    AETHERIS_DEVICE_LIST_ERROR_SERIALIZATION_ERROR = -5,
    AETHERIS_DEVICE_LIST_ERROR_DESERIALIZATION_ERROR = -6,
    AETHERIS_DEVICE_LIST_ERROR_UNKNOWN = -999
} aetheris_device_list_error_t;

/**
 * @brief Discover all available devices
 * 
 * @param devices Output array of device information (caller must free)
 * @param count Output number of devices discovered
 * @return AETHERIS_DEVICE_LIST_SUCCESS on success, error code on failure
 */
aetheris_device_list_error_t aetheris_discover_devices(
    aetheris_device_info_t** devices,
    size_t* count
);

/**
 * @brief Get devices by type
 * 
 * @param device_type Device type to filter by
 * @param devices Output array of device information (caller must free)
 * @param count Output number of devices found
 * @return AETHERIS_DEVICE_LIST_SUCCESS on success, error code on failure
 */
aetheris_device_list_error_t aetheris_get_devices_by_type(
    aetheris_device_type_t device_type,
    aetheris_device_info_t** devices,
    size_t* count
);

/**
 * @brief Get devices by provider
 * 
 * @param provider_type Provider type to filter by
 * @param devices Output array of device information (caller must free)
 * @param count Output number of devices found
 * @return AETHERIS_DEVICE_LIST_SUCCESS on success, error code on failure
 */
aetheris_device_list_error_t aetheris_get_devices_by_provider(
    aetheris_provider_type_t provider_type,
    aetheris_device_info_t** devices,
    size_t* count
);

/**
 * @brief Get a specific device by ID
 * 
 * @param device_id Device identifier
 * @param device Output device information
 * @return AETHERIS_DEVICE_LIST_SUCCESS on success, error code on failure
 */
aetheris_device_list_error_t aetheris_get_device_by_id(
    const char* device_id,
    aetheris_device_info_t* device
);

/**
 * @brief Get provider status for all providers
 * 
 * @param statuses Output array of provider status (caller must free)
 * @param count Output number of providers
 * @return AETHERIS_DEVICE_LIST_SUCCESS on success, error code on failure
 */
aetheris_device_list_error_t aetheris_get_provider_status(
    aetheris_provider_status_t** statuses,
    size_t* count
);

/**
 * @brief Check if a provider is available
 * 
 * @param provider_type Provider type to check
 * @param is_available Output availability status
 * @return AETHERIS_DEVICE_LIST_SUCCESS on success, error code on failure
 */
aetheris_device_list_error_t aetheris_is_provider_available(
    aetheris_provider_type_t provider_type,
    bool* is_available
);

/**
 * @brief Set the default provider
 * 
 * @param provider_type Provider type to set as default
 * @return AETHERIS_DEVICE_LIST_SUCCESS on success, error code on failure
 */
aetheris_device_list_error_t aetheris_set_default_provider(
    aetheris_provider_type_t provider_type
);

/**
 * @brief Get the current default provider
 * 
 * @param provider_type Output current default provider
 * @return AETHERIS_DEVICE_LIST_SUCCESS on success, error code on failure
 */
aetheris_device_list_error_t aetheris_get_default_provider(
    aetheris_provider_type_t* provider_type
);

/**
 * @brief Free device information array
 * 
 * @param devices Device information array to free
 */
void aetheris_free_device_info_array(aetheris_device_info_t* devices);

/**
 * @brief Free provider status array
 * 
 * @param statuses Provider status array to free
 */
void aetheris_free_provider_status_array(aetheris_provider_status_t* statuses);

/**
 * @brief Get error message for error code
 * 
 * @param error Error code
 * @return Error message string
 */
const char* aetheris_device_list_error_message(aetheris_device_list_error_t error);

/**
 * @brief Get device type string
 * 
 * @param device_type Device type enum
 * @return Device type string
 */
const char* aetheris_device_type_to_string(aetheris_device_type_t device_type);

/**
 * @brief Get provider type string
 * 
 * @param provider_type Provider type enum
 * @return Provider type string
 */
const char* aetheris_provider_type_to_string(aetheris_provider_type_t provider_type);

/**
 * @brief Parse device type from string
 * 
 * @param device_type_str Device type string
 * @param device_type Output device type enum
 * @return AETHERIS_DEVICE_LIST_SUCCESS on success, error code on failure
 */
aetheris_device_list_error_t aetheris_parse_device_type(
    const char* device_type_str,
    aetheris_device_type_t* device_type
);

/**
 * @brief Parse provider type from string
 * 
 * @param provider_type_str Provider type string
 * @param provider_type Output provider type enum
 * @return AETHERIS_DEVICE_LIST_SUCCESS on success, error code on failure
 */
aetheris_device_list_error_t aetheris_parse_provider_type(
    const char* provider_type_str,
    aetheris_provider_type_t* provider_type
);

#ifdef __cplusplus
}
#endif

#endif // AETHERIS_DEVICES_LIST_H
