/**
 * @file devices_list.c
 * @brief C FFI implementation for Aetheris Device Listing and Provider Management
 * 
 * This file provides the C implementation for device discovery, listing,
 * and provider management operations.
 */

#include "devices_list.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Forward declarations for Rust FFI functions
extern int32_t rust_discover_devices(
    char** device_ids,
    uint32_t* device_types,
    char** device_paths,
    char** capabilities,
    uint32_t* providers,
    bool* is_available,
    char** metadata,
    size_t* count
);

extern int32_t rust_get_devices_by_type(
    uint32_t device_type,
    char** device_ids,
    uint32_t* device_types,
    char** device_paths,
    char** capabilities,
    uint32_t* providers,
    bool* is_available,
    char** metadata,
    size_t* count
);

extern int32_t rust_get_devices_by_provider(
    uint32_t provider_type,
    char** device_ids,
    uint32_t* device_types,
    char** device_paths,
    char** capabilities,
    uint32_t* providers,
    bool* is_available,
    char** metadata,
    size_t* count
);

extern int32_t rust_get_device_by_id(
    const char* device_id,
    char* device_id_out,
    uint32_t* device_type,
    char* device_path,
    char* capabilities,
    uint32_t* provider,
    bool* is_available,
    char* metadata
);

extern int32_t rust_get_provider_status(
    uint32_t* provider_ids,
    bool* is_available,
    size_t* device_counts,
    char** last_errors,
    char** capabilities,
    size_t* count
);

extern int32_t rust_is_provider_available(uint32_t provider_type, bool* is_available);
extern int32_t rust_set_default_provider(uint32_t provider_type);
extern int32_t rust_get_default_provider(uint32_t* provider_type);
extern void rust_free_device_arrays(
    char* device_ids,
    uint32_t* device_types,
    char* device_paths,
    char* capabilities,
    uint32_t* providers,
    bool* is_available,
    char* metadata
);
extern void rust_free_provider_arrays(
    uint32_t* provider_ids,
    bool* is_available,
    size_t* device_counts,
    char* last_errors,
    char* capabilities
);
extern const char* rust_device_list_error_message(int32_t error);

// Helper function to convert error codes
static aetheris_device_list_error_t convert_error(int32_t rust_error) {
    switch (rust_error) {
        case 0: return AETHERIS_DEVICE_LIST_SUCCESS;
        case -1: return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
        case -2: return AETHERIS_DEVICE_LIST_ERROR_DEVICE_NOT_FOUND;
        case -3: return AETHERIS_DEVICE_LIST_ERROR_PROVIDER_NOT_FOUND;
        case -4: return AETHERIS_DEVICE_LIST_ERROR_DISCOVERY_FAILED;
        case -5: return AETHERIS_DEVICE_LIST_ERROR_SERIALIZATION_ERROR;
        case -6: return AETHERIS_DEVICE_LIST_ERROR_DESERIALIZATION_ERROR;
        default: return AETHERIS_DEVICE_LIST_ERROR_UNKNOWN;
    }
}

// Helper function to convert device types
static aetheris_device_type_t convert_device_type(uint32_t rust_type) {
    switch (rust_type) {
        case 0: return AETHERIS_DEVICE_TYPE_CAMERA;
        case 1: return AETHERIS_DEVICE_TYPE_MICROPHONE;
        case 2: return AETHERIS_DEVICE_TYPE_GPIO;
        case 3: return AETHERIS_DEVICE_TYPE_ADC;
        case 4: return AETHERIS_DEVICE_TYPE_ACTUATOR;
        default: return AETHERIS_DEVICE_TYPE_CAMERA;
    }
}

// Helper function to convert provider types
static aetheris_provider_type_t convert_provider_type(uint32_t rust_type) {
    switch (rust_type) {
        case 0: return AETHERIS_PROVIDER_DETERMINISTIC;
        case 1: return AETHERIS_PROVIDER_LINUX;
        case 2: return AETHERIS_PROVIDER_CUSTOM;
        default: return AETHERIS_PROVIDER_DETERMINISTIC;
    }
}

aetheris_device_list_error_t aetheris_discover_devices(
    aetheris_device_info_t** devices,
    size_t* count
) {
    if (!devices || !count) {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    char* device_ids = NULL;
    uint32_t* device_types = NULL;
    char* device_paths = NULL;
    char* capabilities = NULL;
    uint32_t* providers = NULL;
    bool* is_available = NULL;
    char* metadata = NULL;
    size_t device_count = 0;

    int32_t result = rust_discover_devices(
        &device_ids,
        device_types,
        &device_paths,
        &capabilities,
        providers,
        is_available,
        &metadata,
        &device_count
    );

    if (result != 0) {
        return convert_error(result);
    }

    if (device_count == 0) {
        *devices = NULL;
        *count = 0;
        return AETHERIS_DEVICE_LIST_SUCCESS;
    }

    // Allocate memory for device info array
    aetheris_device_info_t* device_array = malloc(device_count * sizeof(aetheris_device_info_t));
    if (!device_array) {
        rust_free_device_arrays(device_ids, device_types, device_paths, capabilities, providers, is_available, metadata);
        return AETHERIS_DEVICE_LIST_ERROR_UNKNOWN;
    }

    // Copy data from Rust arrays
    for (size_t i = 0; i < device_count; i++) {
        strncpy(device_array[i].device_id, &device_ids[i * 128], 127);
        device_array[i].device_id[127] = '\0';
        
        device_array[i].device_type = convert_device_type(device_types[i]);
        
        strncpy(device_array[i].device_path, &device_paths[i * 256], 255);
        device_array[i].device_path[255] = '\0';
        
        strncpy(device_array[i].capabilities, &capabilities[i * 512], 511);
        device_array[i].capabilities[511] = '\0';
        
        device_array[i].provider = convert_provider_type(providers[i]);
        device_array[i].is_available = is_available[i];
        
        strncpy(device_array[i].metadata, &metadata[i * 1024], 1023);
        device_array[i].metadata[1023] = '\0';
    }

    rust_free_device_arrays(device_ids, device_types, device_paths, capabilities, providers, is_available, metadata);

    *devices = device_array;
    *count = device_count;
    return AETHERIS_DEVICE_LIST_SUCCESS;
}

aetheris_device_list_error_t aetheris_get_devices_by_type(
    aetheris_device_type_t device_type,
    aetheris_device_info_t** devices,
    size_t* count
) {
    if (!devices || !count) {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    uint32_t rust_device_type = (uint32_t)device_type;
    char* device_ids = NULL;
    uint32_t* device_types = NULL;
    char* device_paths = NULL;
    char* capabilities = NULL;
    uint32_t* providers = NULL;
    bool* is_available = NULL;
    char* metadata = NULL;
    size_t device_count = 0;

    int32_t result = rust_get_devices_by_type(
        rust_device_type,
        &device_ids,
        device_types,
        &device_paths,
        &capabilities,
        providers,
        is_available,
        &metadata,
        &device_count
    );

    if (result != 0) {
        return convert_error(result);
    }

    if (device_count == 0) {
        *devices = NULL;
        *count = 0;
        return AETHERIS_DEVICE_LIST_SUCCESS;
    }

    // Allocate memory for device info array
    aetheris_device_info_t* device_array = malloc(device_count * sizeof(aetheris_device_info_t));
    if (!device_array) {
        rust_free_device_arrays(device_ids, device_types, device_paths, capabilities, providers, is_available, metadata);
        return AETHERIS_DEVICE_LIST_ERROR_UNKNOWN;
    }

    // Copy data from Rust arrays (same as discover_devices)
    for (size_t i = 0; i < device_count; i++) {
        strncpy(device_array[i].device_id, &device_ids[i * 128], 127);
        device_array[i].device_id[127] = '\0';
        
        device_array[i].device_type = convert_device_type(device_types[i]);
        
        strncpy(device_array[i].device_path, &device_paths[i * 256], 255);
        device_array[i].device_path[255] = '\0';
        
        strncpy(device_array[i].capabilities, &capabilities[i * 512], 511);
        device_array[i].capabilities[511] = '\0';
        
        device_array[i].provider = convert_provider_type(providers[i]);
        device_array[i].is_available = is_available[i];
        
        strncpy(device_array[i].metadata, &metadata[i * 1024], 1023);
        device_array[i].metadata[1023] = '\0';
    }

    rust_free_device_arrays(device_ids, device_types, device_paths, capabilities, providers, is_available, metadata);

    *devices = device_array;
    *count = device_count;
    return AETHERIS_DEVICE_LIST_SUCCESS;
}

aetheris_device_list_error_t aetheris_get_devices_by_provider(
    aetheris_provider_type_t provider_type,
    aetheris_device_info_t** devices,
    size_t* count
) {
    if (!devices || !count) {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    uint32_t rust_provider_type = (uint32_t)provider_type;
    char* device_ids = NULL;
    uint32_t* device_types = NULL;
    char* device_paths = NULL;
    char* capabilities = NULL;
    uint32_t* providers = NULL;
    bool* is_available = NULL;
    char* metadata = NULL;
    size_t device_count = 0;

    int32_t result = rust_get_devices_by_provider(
        rust_provider_type,
        &device_ids,
        device_types,
        &device_paths,
        &capabilities,
        providers,
        is_available,
        &metadata,
        &device_count
    );

    if (result != 0) {
        return convert_error(result);
    }

    if (device_count == 0) {
        *devices = NULL;
        *count = 0;
        return AETHERIS_DEVICE_LIST_SUCCESS;
    }

    // Allocate memory for device info array
    aetheris_device_info_t* device_array = malloc(device_count * sizeof(aetheris_device_info_t));
    if (!device_array) {
        rust_free_device_arrays(device_ids, device_types, device_paths, capabilities, providers, is_available, metadata);
        return AETHERIS_DEVICE_LIST_ERROR_UNKNOWN;
    }

    // Copy data from Rust arrays (same as discover_devices)
    for (size_t i = 0; i < device_count; i++) {
        strncpy(device_array[i].device_id, &device_ids[i * 128], 127);
        device_array[i].device_id[127] = '\0';
        
        device_array[i].device_type = convert_device_type(device_types[i]);
        
        strncpy(device_array[i].device_path, &device_paths[i * 256], 255);
        device_array[i].device_path[255] = '\0';
        
        strncpy(device_array[i].capabilities, &capabilities[i * 512], 511);
        device_array[i].capabilities[511] = '\0';
        
        device_array[i].provider = convert_provider_type(providers[i]);
        device_array[i].is_available = is_available[i];
        
        strncpy(device_array[i].metadata, &metadata[i * 1024], 1023);
        device_array[i].metadata[1023] = '\0';
    }

    rust_free_device_arrays(device_ids, device_types, device_paths, capabilities, providers, is_available, metadata);

    *devices = device_array;
    *count = device_count;
    return AETHERIS_DEVICE_LIST_SUCCESS;
}

aetheris_device_list_error_t aetheris_get_device_by_id(
    const char* device_id,
    aetheris_device_info_t* device
) {
    if (!device_id || !device) {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    char device_id_out[128];
    uint32_t device_type;
    char device_path[256];
    char capabilities[512];
    uint32_t provider;
    bool is_available;
    char metadata[1024];

    int32_t result = rust_get_device_by_id(
        device_id,
        device_id_out,
        &device_type,
        device_path,
        capabilities,
        &provider,
        &is_available,
        metadata
    );

    if (result != 0) {
        return convert_error(result);
    }

    // Copy data to output structure
    strncpy(device->device_id, device_id_out, 127);
    device->device_id[127] = '\0';
    
    device->device_type = convert_device_type(device_type);
    
    strncpy(device->device_path, device_path, 255);
    device->device_path[255] = '\0';
    
    strncpy(device->capabilities, capabilities, 511);
    device->capabilities[511] = '\0';
    
    device->provider = convert_provider_type(provider);
    device->is_available = is_available;
    
    strncpy(device->metadata, metadata, 1023);
    device->metadata[1023] = '\0';

    return AETHERIS_DEVICE_LIST_SUCCESS;
}

aetheris_device_list_error_t aetheris_get_provider_status(
    aetheris_provider_status_t** statuses,
    size_t* count
) {
    if (!statuses || !count) {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    uint32_t* provider_ids = NULL;
    bool* is_available = NULL;
    size_t* device_counts = NULL;
    char* last_errors = NULL;
    char* capabilities = NULL;
    size_t provider_count = 0;

    int32_t result = rust_get_provider_status(
        provider_ids,
        is_available,
        device_counts,
        &last_errors,
        &capabilities,
        &provider_count
    );

    if (result != 0) {
        return convert_error(result);
    }

    if (provider_count == 0) {
        *statuses = NULL;
        *count = 0;
        return AETHERIS_DEVICE_LIST_SUCCESS;
    }

    // Allocate memory for provider status array
    aetheris_provider_status_t* status_array = malloc(provider_count * sizeof(aetheris_provider_status_t));
    if (!status_array) {
        rust_free_provider_arrays(provider_ids, is_available, device_counts, last_errors, capabilities);
        return AETHERIS_DEVICE_LIST_ERROR_UNKNOWN;
    }

    // Copy data from Rust arrays
    for (size_t i = 0; i < provider_count; i++) {
        status_array[i].provider_id = convert_provider_type(provider_ids[i]);
        status_array[i].is_available = is_available[i];
        status_array[i].device_count = device_counts[i];
        
        strncpy(status_array[i].last_error, &last_errors[i * 256], 255);
        status_array[i].last_error[255] = '\0';
        
        strncpy(status_array[i].capabilities, &capabilities[i * 512], 511);
        status_array[i].capabilities[511] = '\0';
    }

    rust_free_provider_arrays(provider_ids, is_available, device_counts, last_errors, capabilities);

    *statuses = status_array;
    *count = provider_count;
    return AETHERIS_DEVICE_LIST_SUCCESS;
}

aetheris_device_list_error_t aetheris_is_provider_available(
    aetheris_provider_type_t provider_type,
    bool* is_available
) {
    if (!is_available) {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    uint32_t rust_provider_type = (uint32_t)provider_type;
    return convert_error(rust_is_provider_available(rust_provider_type, is_available));
}

aetheris_device_list_error_t aetheris_set_default_provider(
    aetheris_provider_type_t provider_type
) {
    uint32_t rust_provider_type = (uint32_t)provider_type;
    return convert_error(rust_set_default_provider(rust_provider_type));
}

aetheris_device_list_error_t aetheris_get_default_provider(
    aetheris_provider_type_t* provider_type
) {
    if (!provider_type) {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    uint32_t rust_provider_type;
    int32_t result = rust_get_default_provider(&rust_provider_type);
    if (result == 0) {
        *provider_type = convert_provider_type(rust_provider_type);
    }
    return convert_error(result);
}

void aetheris_free_device_info_array(aetheris_device_info_t* devices) {
    if (devices) {
        free(devices);
    }
}

void aetheris_free_provider_status_array(aetheris_provider_status_t* statuses) {
    if (statuses) {
        free(statuses);
    }
}

const char* aetheris_device_list_error_message(aetheris_device_list_error_t error) {
    return rust_device_list_error_message((int32_t)error);
}

const char* aetheris_device_type_to_string(aetheris_device_type_t device_type) {
    switch (device_type) {
        case AETHERIS_DEVICE_TYPE_CAMERA: return "camera";
        case AETHERIS_DEVICE_TYPE_MICROPHONE: return "microphone";
        case AETHERIS_DEVICE_TYPE_GPIO: return "gpio";
        case AETHERIS_DEVICE_TYPE_ADC: return "adc";
        case AETHERIS_DEVICE_TYPE_ACTUATOR: return "actuator";
        default: return "unknown";
    }
}

const char* aetheris_provider_type_to_string(aetheris_provider_type_t provider_type) {
    switch (provider_type) {
        case AETHERIS_PROVIDER_DETERMINISTIC: return "deterministic";
        case AETHERIS_PROVIDER_LINUX: return "linux";
        case AETHERIS_PROVIDER_CUSTOM: return "custom";
        default: return "unknown";
    }
}

aetheris_device_list_error_t aetheris_parse_device_type(
    const char* device_type_str,
    aetheris_device_type_t* device_type
) {
    if (!device_type_str || !device_type) {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    if (strcmp(device_type_str, "camera") == 0) {
        *device_type = AETHERIS_DEVICE_TYPE_CAMERA;
    } else if (strcmp(device_type_str, "microphone") == 0) {
        *device_type = AETHERIS_DEVICE_TYPE_MICROPHONE;
    } else if (strcmp(device_type_str, "gpio") == 0) {
        *device_type = AETHERIS_DEVICE_TYPE_GPIO;
    } else if (strcmp(device_type_str, "adc") == 0) {
        *device_type = AETHERIS_DEVICE_TYPE_ADC;
    } else if (strcmp(device_type_str, "actuator") == 0) {
        *device_type = AETHERIS_DEVICE_TYPE_ACTUATOR;
    } else {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    return AETHERIS_DEVICE_LIST_SUCCESS;
}

aetheris_device_list_error_t aetheris_parse_provider_type(
    const char* provider_type_str,
    aetheris_provider_type_t* provider_type
) {
    if (!provider_type_str || !provider_type) {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    if (strcmp(provider_type_str, "deterministic") == 0 || strcmp(provider_type_str, "det") == 0) {
        *provider_type = AETHERIS_PROVIDER_DETERMINISTIC;
    } else if (strcmp(provider_type_str, "linux") == 0) {
        *provider_type = AETHERIS_PROVIDER_LINUX;
    } else if (strcmp(provider_type_str, "custom") == 0) {
        *provider_type = AETHERIS_PROVIDER_CUSTOM;
    } else {
        return AETHERIS_DEVICE_LIST_ERROR_INVALID_PARAM;
    }

    return AETHERIS_DEVICE_LIST_SUCCESS;
}
