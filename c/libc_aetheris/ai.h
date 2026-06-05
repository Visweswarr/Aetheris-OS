/**
 * @file ai.h
 * @brief Aetheris AI Service C FFI
 * 
 * This header provides C bindings for the Aetheris AI service,
 * enabling integration with C/C++ applications.
 */

#ifndef AETHERIS_AI_H
#define AETHERIS_AI_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief AI service handle
 */
typedef struct aetheris_ai_service aetheris_ai_service_t;

/**
 * @brief AI session handle
 */
typedef struct aetheris_ai_session aetheris_ai_session_t;

/**
 * @brief AI event handle
 */
typedef struct aetheris_ai_event aetheris_ai_event_t;

/**
 * @brief AI model information
 */
typedef struct {
    char name[256];
    char path[512];
    char model_type[64];
    char model_size[64];
    uint32_t input_shape[8];
    uint32_t output_shape[8];
    uint64_t size_bytes;
    bool available;
} aetheris_ai_model_info_t;

/**
 * @brief AI configuration
 */
typedef struct {
    char model_paths[16][512];
    uint32_t num_model_paths;
    bool enable_gpu;
    uint32_t max_latency_ms;
    uint32_t target_fps;
    uint32_t audio_buffer_size;
    bool enable_monitoring;
    uint64_t default_seed;
    bool enable_deterministic;
    uint32_t replay_buffer_size;
} aetheris_ai_config_t;

/**
 * @brief Vision pipeline configuration
 */
typedef struct {
    uint32_t fps;
    char model_name[256];
    char model_path[512];
    uint32_t input_width;
    uint32_t input_height;
    float confidence_threshold;
    float nms_threshold;
    uint32_t num_classes;
    bool enable_encoding;
    char encoder_type[32];
    uint8_t quality;
    uint32_t bitrate;
    uint64_t seed;
    bool enable_deterministic;
} aetheris_ai_vision_config_t;

/**
 * @brief Audio pipeline configuration
 */
typedef struct {
    uint32_t sample_rate;
    uint16_t channels;
    char model_name[256];
    char model_path[512];
    char model_size[32];
    char language[16];
    uint32_t threads;
    bool enable_gpu;
    bool enable_vad;
    float vad_threshold;
    uint64_t min_speech_duration_ms;
    uint64_t min_silence_duration_ms;
    bool enable_punctuation;
    bool enable_capitalization;
    float confidence_threshold;
    uint64_t seed;
    bool enable_deterministic;
} aetheris_ai_audio_config_t;

/**
 * @brief Detection result
 */
typedef struct {
    uint32_t class_id;
    char class_name[64];
    float confidence;
    float x, y, width, height;
} aetheris_ai_detection_t;

/**
 * @brief Transcript result
 */
typedef struct {
    char text[1024];
    char language[16];
    float confidence;
    float start_time;
    float end_time;
} aetheris_ai_transcript_t;

/**
 * @brief VAD result
 */
typedef struct {
    char state[16];
    float confidence;
    float audio_level_db;
    float duration_ms;
} aetheris_ai_vad_t;

/**
 * @brief AI statistics
 */
typedef struct {
    // Vision stats
    uint64_t frames_processed;
    uint64_t total_detections;
    double avg_inference_time_ms;
    double avg_fps;
    uint64_t vision_errors;
    
    // Audio stats
    uint64_t samples_processed;
    uint64_t total_transcripts;
    uint64_t total_vad_activations;
    double avg_audio_inference_time_ms;
    double avg_audio_level_db;
    uint64_t audio_errors;
    
    // System stats
    double memory_mb;
    double cpu_percent;
    uint32_t active_pipelines;
    uint64_t cache_hits;
    uint64_t cache_misses;
} aetheris_ai_stats_t;

/**
 * @brief Error codes
 */
typedef enum {
    AETHERIS_AI_SUCCESS = 0,
    AETHERIS_AI_ERROR_INVALID_PARAM = -1,
    AETHERIS_AI_ERROR_INITIALIZATION = -2,
    AETHERIS_AI_ERROR_MODEL_LOADING = -3,
    AETHERIS_AI_ERROR_INFERENCE = -4,
    AETHERIS_AI_ERROR_BACKEND = -5,
    AETHERIS_AI_ERROR_DEVICE = -6,
    AETHERIS_AI_ERROR_CAPABILITY = -7,
    AETHERIS_AI_ERROR_POLICY = -8,
    AETHERIS_AI_ERROR_REPLAY = -9,
    AETHERIS_AI_ERROR_ENCODING = -10,
    AETHERIS_AI_ERROR_IO = -11,
    AETHERIS_AI_ERROR_SERIALIZATION = -12,
    AETHERIS_AI_ERROR_TIMEOUT = -13,
    AETHERIS_AI_ERROR_RESOURCE = -14,
    AETHERIS_AI_ERROR_VALIDATION = -15,
    AETHERIS_AI_ERROR_INTERNAL = -16
} aetheris_ai_error_t;

/**
 * @brief Initialize AI service
 * 
 * @param config AI configuration
 * @param service Output service handle
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_service_init(
    const aetheris_ai_config_t* config,
    aetheris_ai_service_t** service
);

/**
 * @brief Destroy AI service
 * 
 * @param service Service handle
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_service_destroy(aetheris_ai_service_t* service);

/**
 * @brief Get AI service statistics
 * 
 * @param service Service handle
 * @param stats Output statistics
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_get_stats(
    const aetheris_ai_service_t* service,
    aetheris_ai_stats_t* stats
);

/**
 * @brief Start vision pipeline
 * 
 * @param service Service handle
 * @param device_path Camera device path
 * @param config Vision configuration
 * @param session Output session handle
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_start_vision_pipeline(
    aetheris_ai_service_t* service,
    const char* device_path,
    const aetheris_ai_vision_config_t* config,
    aetheris_ai_session_t** session
);

/**
 * @brief Start audio pipeline
 * 
 * @param service Service handle
 * @param device_path Audio device path
 * @param config Audio configuration
 * @param session Output session handle
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_start_audio_pipeline(
    aetheris_ai_service_t* service,
    const char* device_path,
    const aetheris_ai_audio_config_t* config,
    aetheris_ai_session_t** session
);

/**
 * @brief Stop AI pipeline
 * 
 * @param session Session handle
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_stop_pipeline(aetheris_ai_session_t* session);

/**
 * @brief Get available models
 * 
 * @param service Service handle
 * @param models Output model array
 * @param max_models Maximum number of models
 * @param num_models Output number of models
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_get_available_models(
    const aetheris_ai_service_t* service,
    aetheris_ai_model_info_t* models,
    size_t max_models,
    size_t* num_models
);

/**
 * @brief Process frame (vision)
 * 
 * @param session Session handle
 * @param frame_data Frame data
 * @param frame_size Frame data size
 * @param width Frame width
 * @param height Frame height
 * @param detections Output detections array
 * @param max_detections Maximum number of detections
 * @param num_detections Output number of detections
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_process_frame(
    aetheris_ai_session_t* session,
    const uint8_t* frame_data,
    size_t frame_size,
    uint32_t width,
    uint32_t height,
    aetheris_ai_detection_t* detections,
    size_t max_detections,
    size_t* num_detections
);

/**
 * @brief Process audio (audio)
 * 
 * @param session Session handle
 * @param audio_data Audio data
 * @param audio_size Audio data size
 * @param sample_rate Sample rate
 * @param channels Number of channels
 * @param transcript Output transcript
 * @param vad Output VAD result
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_process_audio(
    aetheris_ai_session_t* session,
    const float* audio_data,
    size_t audio_size,
    uint32_t sample_rate,
    uint16_t channels,
    aetheris_ai_transcript_t* transcript,
    aetheris_ai_vad_t* vad
);

/**
 * @brief Replay AI events
 * 
 * @param service Service handle
 * @param snapshot_id Snapshot ID
 * @param topic Event topic
 * @param check_determinism Enable deterministic checking
 * @param events Output events array
 * @param max_events Maximum number of events
 * @param num_events Output number of events
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_replay_events(
    const aetheris_ai_service_t* service,
    const char* snapshot_id,
    const char* topic,
    bool check_determinism,
    aetheris_ai_event_t* events,
    size_t max_events,
    size_t* num_events
);

/**
 * @brief Get AI event data
 * 
 * @param event Event handle
 * @param data Output data buffer
 * @param max_size Maximum data size
 * @param actual_size Output actual data size
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_event_get_data(
    const aetheris_ai_event_t* event,
    uint8_t* data,
    size_t max_size,
    size_t* actual_size
);

/**
 * @brief Get AI event type
 * 
 * @param event Event handle
 * @param event_type Output event type string
 * @param max_size Maximum string size
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_event_get_type(
    const aetheris_ai_event_t* event,
    char* event_type,
    size_t max_size
);

/**
 * @brief Get AI event timestamp
 * 
 * @param event Event handle
 * @param timestamp Output timestamp
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_event_get_timestamp(
    const aetheris_ai_event_t* event,
    uint64_t* timestamp
);

/**
 * @brief Destroy AI event
 * 
 * @param event Event handle
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_event_destroy(aetheris_ai_event_t* event);

/**
 * @brief Check AI capability
 * 
 * @param service Service handle
 * @param capability Capability string
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_check_capability(
    const aetheris_ai_service_t* service,
    const char* capability
);

/**
 * @brief Set AI policy
 * 
 * @param service Service handle
 * @param policy_json Policy JSON string
 * @return Error code
 */
aetheris_ai_error_t aetheris_ai_set_policy(
    aetheris_ai_service_t* service,
    const char* policy_json
);

/**
 * @brief Get AI error string
 * 
 * @param error Error code
 * @return Error string
 */
const char* aetheris_ai_error_string(aetheris_ai_error_t error);

/**
 * @brief Get AI version
 * 
 * @return Version string
 */
const char* aetheris_ai_version(void);

#ifdef __cplusplus
}
#endif

#endif /* AETHERIS_AI_H */
