/**
 * @file devices_ble.h
 * @brief C FFI interface for Aetheris BLE Device Runtime
 * 
 * This header provides a thin C interface for BLE operations,
 * including scanning, connecting, GATT read/write, and notification
 * subscriptions with capability gating and deterministic event handling.
 */

#ifndef AETHERIS_DEVICES_BLE_H
#define AETHERIS_DEVICES_BLE_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief BLE scan filters
 */
typedef struct {
    char name_prefix[64];
    char service_uuids[256];  // Comma-separated UUIDs
    int16_t rssi_threshold;
    uint32_t timeout_ms;
} aetheris_ble_scan_filters_t;

/**
 * @brief BLE device information
 */
typedef struct {
    char address[18];  // MAC address string
    char name[64];
    int16_t rssi;
    char service_uuids[256];  // Comma-separated UUIDs
    uint64_t last_seen_timestamp;
} aetheris_ble_device_info_t;

/**
 * @brief BLE notification data
 */
typedef struct {
    char notify_handle[64];
    char service_uuid[8];
    char char_uuid[8];
    uint8_t* data;
    size_t data_size;
    uint64_t timestamp;
} aetheris_ble_notification_t;

/**
 * @brief Error codes for BLE operations
 */
typedef enum {
    AETHERIS_BLE_SUCCESS = 0,
    AETHERIS_BLE_ERROR_INVALID_PARAM = -1,
    AETHERIS_BLE_ERROR_DEVICE_NOT_FOUND = -2,
    AETHERIS_BLE_ERROR_CONNECTION_FAILED = -3,
    AETHERIS_BLE_ERROR_GATT_FAILED = -4,
    AETHERIS_BLE_ERROR_NOTIFICATION_FAILED = -5,
    AETHERIS_BLE_ERROR_INVALID_CAPABILITY = -6,
    AETHERIS_BLE_ERROR_DAO_POLICY_DENIED = -7,
    AETHERIS_BLE_ERROR_SCAN_FAILED = -8,
    AETHERIS_BLE_ERROR_UNKNOWN = -999
} aetheris_ble_error_t;

/**
 * @brief Initialize BLE runtime
 * 
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_init(void);

/**
 * @brief Shutdown BLE runtime
 * 
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_shutdown(void);

/**
 * @brief Start BLE scan
 * 
 * @param session_id Session identifier
 * @param caps Capability string (comma-separated)
 * @param filters Scan filters
 * @param scan_handle Output scan handle (must be at least 64 bytes)
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_scan_start(
    const char* session_id,
    const char* caps,
    const aetheris_ble_scan_filters_t* filters,
    char* scan_handle
);

/**
 * @brief Stop BLE scan
 * 
 * @param scan_handle Scan handle
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_scan_stop(const char* scan_handle);

/**
 * @brief List discovered BLE devices
 * 
 * @param devices Output array of device info
 * @param max_devices Maximum number of devices to return
 * @param device_count Output number of devices found
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_list_devices(
    aetheris_ble_device_info_t* devices,
    size_t max_devices,
    size_t* device_count
);

/**
 * @brief Connect to BLE device
 * 
 * @param session_id Session identifier
 * @param caps Capability string (comma-separated)
 * @param address Device MAC address
 * @param conn_handle Output connection handle (must be at least 64 bytes)
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_connect(
    const char* session_id,
    const char* caps,
    const char* address,
    char* conn_handle
);

/**
 * @brief Disconnect from BLE device
 * 
 * @param conn_handle Connection handle
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_disconnect(const char* conn_handle);

/**
 * @brief Read GATT characteristic
 * 
 * @param conn_handle Connection handle
 * @param service_uuid Service UUID
 * @param char_uuid Characteristic UUID
 * @param data Output data buffer
 * @param data_size Input/output data buffer size
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_gatt_read(
    const char* conn_handle,
    const char* service_uuid,
    const char* char_uuid,
    uint8_t* data,
    size_t* data_size
);

/**
 * @brief Write GATT characteristic
 * 
 * @param conn_handle Connection handle
 * @param service_uuid Service UUID
 * @param char_uuid Characteristic UUID
 * @param data Data to write
 * @param data_size Data size
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_gatt_write(
    const char* conn_handle,
    const char* service_uuid,
    const char* char_uuid,
    const uint8_t* data,
    size_t data_size
);

/**
 * @brief Subscribe to GATT notifications
 * 
 * @param conn_handle Connection handle
 * @param service_uuid Service UUID
 * @param char_uuid Characteristic UUID
 * @param notify_handle Output notification handle (must be at least 64 bytes)
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_subscribe(
    const char* conn_handle,
    const char* service_uuid,
    const char* char_uuid,
    char* notify_handle
);

/**
 * @brief Unsubscribe from GATT notifications
 * 
 * @param notify_handle Notification handle
 * @return AETHERIS_BLE_SUCCESS on success, error code on failure
 */
aetheris_ble_error_t aetheris_ble_unsubscribe(const char* notify_handle);

/**
 * @brief Get BLE error message
 * 
 * @param error Error code
 * @return Error message string
 */
const char* aetheris_ble_error_message(aetheris_ble_error_t error);

/**
 * @brief Set default scan filters
 * 
 * @param filters Output filters
 */
void aetheris_ble_scan_filters_default(aetheris_ble_scan_filters_t* filters);

#ifdef __cplusplus
}
#endif

#endif // AETHERIS_DEVICES_BLE_H
