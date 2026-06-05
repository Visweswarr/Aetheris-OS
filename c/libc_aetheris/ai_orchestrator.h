/**
 * @file ai_orchestrator.h
 * @brief AI Orchestrator & Assistant C FFI - Phase 5
 * 
 * C language bindings for AI Orchestrator and Assistant functionality.
 * Provides FFI-safe wrappers for Rust AI services.
 * 
 * References:
 * - ONNX Runtime: Model loading and inference orchestration
 * - Whisper.cpp: Audio processing pipeline management
 * - LLaMA.cpp: Text generation pipeline coordination
 * - Transformers: Multi-modal model coordination patterns
 * - VLLM: High-throughput inference orchestration
 * - OpenVINO: Intel hardware acceleration orchestration
 * - TensorRT: NVIDIA hardware acceleration orchestration
 */

#ifndef AETHERIS_AI_ORCHESTRATOR_H
#define AETHERIS_AI_ORCHESTRATOR_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Error codes for AI Orchestrator operations
 */
typedef enum {
    AETHERIS_AI_SUCCESS = 0,
    AETHERIS_AI_ERROR_INVALID_PARAM = 1,
    AETHERIS_AI_ERROR_NOT_INITIALIZED = 2,
    AETHERIS_AI_ERROR_PIPELINE_NOT_FOUND = 3,
    AETHERIS_AI_ERROR_MODEL_NOT_FOUND = 4,
    AETHERIS_AI_ERROR_PLAN_NOT_FOUND = 5,
    AETHERIS_AI_ERROR_TOOL_NOT_FOUND = 6,
    AETHERIS_AI_ERROR_MEMORY_ERROR = 7,
    AETHERIS_AI_ERROR_POLICY_VIOLATION = 8,
    AETHERIS_AI_ERROR_VERIFICATION_FAILED = 9,
    AETHERIS_AI_ERROR_TIMEOUT = 10,
    AETHERIS_AI_ERROR_UNKNOWN = 255
} aetheris_ai_error_t;

/**
 * @brief Pipeline types
 */
typedef enum {
    AETHERIS_AI_PIPELINE_VISION = 0,
    AETHERIS_AI_PIPELINE_AUDIO = 1,
    AETHERIS_AI_PIPELINE_TEXT = 2,
    AETHERIS_AI_PIPELINE_MULTIMODAL = 3
} aetheris_ai_pipeline_type_t;

/**
 * @brief Acceleration backends
 */
typedef enum {
    AETHERIS_AI_BACKEND_CPU = 0,
    AETHERIS_AI_BACKEND_CUDA = 1,
    AETHERIS_AI_BACKEND_INTEL_GPU = 2,
    AETHERIS_AI_BACKEND_METAL = 3,
    AETHERIS_AI_BACKEND_OPENCL = 4,
    AETHERIS_AI_BACKEND_DIRECTML = 5
} aetheris_ai_backend_t;

/**
 * @brief Model formats
 */
typedef enum {
    AETHERIS_AI_MODEL_ONNX = 0,
    AETHERIS_AI_MODEL_PYTORCH = 1,
    AETHERIS_AI_MODEL_TENSORFLOW = 2,
    AETHERIS_AI_MODEL_GGML = 3,
    AETHERIS_AI_MODEL_HUGGINGFACE = 4,
    AETHERIS_AI_MODEL_OPENVINO = 5,
    AETHERIS_AI_MODEL_TENSORRT = 6
} aetheris_ai_model_format_t;

/**
 * @brief Plan status
 */
typedef enum {
    AETHERIS_AI_PLAN_CREATING = 0,
    AETHERIS_AI_PLAN_READY = 1,
    AETHERIS_AI_PLAN_EXECUTING = 2,
    AETHERIS_AI_PLAN_COMPLETED = 3,
    AETHERIS_AI_PLAN_FAILED = 4,
    AETHERIS_AI_PLAN_CANCELLED = 5
} aetheris_ai_plan_status_t;

/**
 * @brief Orchestrator configuration
 */
typedef struct {
    uint32_t max_pipelines;
    uint32_t model_cache_mb;
    bool deterministic;
    bool verify_supply_chain;
    bool enable_monitoring;
} aetheris_ai_orchestrator_config_t;

/**
 * @brief Assistant configuration
 */
typedef struct {
    uint32_t max_context_tokens;
    uint32_t max_tool_invocations;
    bool deterministic;
    bool enable_memory;
    uint64_t memory_retention_secs;
    bool enable_multimodal;
} aetheris_ai_assistant_config_t;

/**
 * @brief Model load configuration
 */
typedef struct {
    const char* model_path;
    aetheris_ai_backend_t backend;
    bool enable_quantization;
    bool enable_optimization;
    bool verify_signatures;
    bool verify_supply_chain;
    bool cache_in_memory;
    bool deterministic;
} aetheris_ai_model_load_config_t;

/**
 * @brief Pipeline information
 */
typedef struct {
    char pipeline_id[64];
    aetheris_ai_pipeline_type_t pipeline_type;
    char model_id[64];
    char model_hash[65]; // SHA-256 hex string
    uint64_t created_at;
    uint64_t last_activity;
} aetheris_ai_pipeline_info_t;

/**
 * @brief Model metadata
 */
typedef struct {
    char id[64];
    char name[128];
    char version[32];
    aetheris_ai_model_format_t format;
    uint64_t size_bytes;
    char hash[65]; // SHA-256 hex string
    char description[256];
    char author[128];
    char license[64];
    uint32_t input_dimensions[8];
    uint32_t output_dimensions[8];
    uint64_t parameter_count;
} aetheris_ai_model_metadata_t;

/**
 * @brief Plan information
 */
typedef struct {
    char plan_id[64];
    char description[256];
    aetheris_ai_plan_status_t status;
    uint64_t created_at;
    uint64_t completed_at;
    uint64_t total_time_us;
    uint32_t steps_count;
} aetheris_ai_plan_info_t;

/**
 * @brief Tool result
 */
typedef struct {
    char tool_id[64];
    bool success;
    char result_data[1024]; // JSON string
    char error_message[256];
    uint64_t execution_time_us;
} aetheris_ai_tool_result_t;

/**
 * @brief Orchestrator statistics
 */
typedef struct {
    uint64_t total_pipelines;
    uint32_t active_pipelines;
    uint64_t total_requests;
    uint64_t avg_latency_us;
    double cache_hit_rate;
    double verification_success_rate;
} aetheris_ai_orchestrator_stats_t;

/**
 * @brief Assistant statistics
 */
typedef struct {
    uint64_t total_plans;
    uint32_t active_plans;
    uint64_t completed_plans;
    uint64_t failed_plans;
    uint64_t total_tool_invocations;
    uint32_t memory_entries;
    uint64_t avg_plan_time_us;
} aetheris_ai_assistant_stats_t;

/**
 * @brief Model statistics
 */
typedef struct {
    uint64_t total_models;
    uint32_t loaded_models;
    uint64_t total_memory_mb;
    double cache_hit_rate;
    uint64_t avg_load_time_us;
    double verification_success_rate;
} aetheris_ai_model_stats_t;

// ============================================================================
// AI Orchestrator Functions
// ============================================================================

/**
 * @brief Initialize AI Orchestrator
 * @param config Orchestrator configuration
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Orchestrator_Init(
    const aetheris_ai_orchestrator_config_t* config
);

/**
 * @brief Start a new AI pipeline
 * @param pipeline_type Type of pipeline to start
 * @param model_id Model identifier
 * @param model_data Model data (can be NULL for stub)
 * @param model_data_size Size of model data
 * @param pipeline_id Output pipeline ID (must be at least 64 bytes)
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Orchestrator_StartPipeline(
    aetheris_ai_pipeline_type_t pipeline_type,
    const char* model_id,
    const uint8_t* model_data,
    size_t model_data_size,
    char* pipeline_id
);

/**
 * @brief Stop a pipeline
 * @param pipeline_id Pipeline identifier
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Orchestrator_StopPipeline(
    const char* pipeline_id
);

/**
 * @brief Get pipeline information
 * @param pipeline_id Pipeline identifier
 * @param info Output pipeline information
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Orchestrator_GetPipeline(
    const char* pipeline_id,
    aetheris_ai_pipeline_info_t* info
);

/**
 * @brief List all active pipelines
 * @param pipelines Output array of pipeline information
 * @param max_pipelines Maximum number of pipelines to return
 * @param actual_count Actual number of pipelines returned
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Orchestrator_ListPipelines(
    aetheris_ai_pipeline_info_t* pipelines,
    uint32_t max_pipelines,
    uint32_t* actual_count
);

/**
 * @brief Attach a model to a pipeline
 * @param pipeline_id Pipeline identifier
 * @param model_id Model identifier
 * @param model_data Model data
 * @param model_data_size Size of model data
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Orchestrator_AttachModel(
    const char* pipeline_id,
    const char* model_id,
    const uint8_t* model_data,
    size_t model_data_size
);

/**
 * @brief Detach a model from a pipeline
 * @param pipeline_id Pipeline identifier
 * @param model_id Model identifier
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Orchestrator_DetachModel(
    const char* pipeline_id,
    const char* model_id
);

/**
 * @brief Get orchestrator statistics
 * @param stats Output statistics
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Orchestrator_GetStats(
    aetheris_ai_orchestrator_stats_t* stats
);

// ============================================================================
// AI Assistant Functions
// ============================================================================

/**
 * @brief Initialize AI Assistant
 * @param config Assistant configuration
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Assistant_Init(
    const aetheris_ai_assistant_config_t* config
);

/**
 * @brief Create a plan for a request
 * @param request Request text
 * @param context Optional context (can be NULL)
 * @param plan_id Output plan ID (must be at least 64 bytes)
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Assistant_Plan(
    const char* request,
    const char* context,
    char* plan_id
);

/**
 * @brief Execute a plan
 * @param plan_id Plan identifier
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Assistant_ExecutePlan(
    const char* plan_id
);

/**
 * @brief Get plan information
 * @param plan_id Plan identifier
 * @param info Output plan information
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Assistant_GetPlan(
    const char* plan_id,
    aetheris_ai_plan_info_t* info
);

/**
 * @brief Cancel a plan
 * @param plan_id Plan identifier
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Assistant_CancelPlan(
    const char* plan_id
);

/**
 * @brief Add memory entry
 * @param content Memory content
 * @param entry_type Entry type ("conversation", "fact", "plan", "result")
 * @param memory_id Output memory ID (must be at least 64 bytes)
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Assistant_AddMemory(
    const char* content,
    const char* entry_type,
    char* memory_id
);

/**
 * @brief Search memory
 * @param query Search query
 * @param results Output array of memory entries (JSON strings)
 * @param max_results Maximum number of results
 * @param actual_count Actual number of results returned
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Assistant_SearchMemory(
    const char* query,
    char* results,
    uint32_t max_results,
    uint32_t* actual_count
);

/**
 * @brief Get assistant statistics
 * @param stats Output statistics
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Assistant_GetStats(
    aetheris_ai_assistant_stats_t* stats
);

// ============================================================================
// Model Loader Functions
// ============================================================================

/**
 * @brief Load a model
 * @param config Model load configuration
 * @param model_id Output model ID (must be at least 64 bytes)
 * @return Error code
 */
aetheris_ai_error_t AetherAI_ModelLoader_LoadModel(
    const aetheris_ai_model_load_config_t* config,
    char* model_id
);

/**
 * @brief Unload a model
 * @param model_id Model identifier
 * @return Error code
 */
aetheris_ai_error_t AetherAI_ModelLoader_UnloadModel(
    const char* model_id
);

/**
 * @brief Get model metadata
 * @param model_id Model identifier
 * @param metadata Output model metadata
 * @return Error code
 */
aetheris_ai_error_t AetherAI_ModelLoader_GetModelMetadata(
    const char* model_id,
    aetheris_ai_model_metadata_t* metadata
);

/**
 * @brief List loaded models
 * @param models Output array of model metadata
 * @param max_models Maximum number of models to return
 * @param actual_count Actual number of models returned
 * @return Error code
 */
aetheris_ai_error_t AetherAI_ModelLoader_ListLoadedModels(
    aetheris_ai_model_metadata_t* models,
    uint32_t max_models,
    uint32_t* actual_count
);

/**
 * @brief Get model statistics
 * @param stats Output statistics
 * @return Error code
 */
aetheris_ai_error_t AetherAI_ModelLoader_GetStats(
    aetheris_ai_model_stats_t* stats
);

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * @brief Get error message for error code
 * @param error Error code
 * @return Error message string
 */
const char* AetherAI_GetErrorMessage(aetheris_ai_error_t error);

/**
 * @brief Get version string
 * @return Version string
 */
const char* AetherAI_GetVersion(void);

/**
 * @brief Shutdown AI services
 * @return Error code
 */
aetheris_ai_error_t AetherAI_Shutdown(void);

#ifdef __cplusplus
}
#endif

#endif // AETHERIS_AI_ORCHESTRATOR_H
