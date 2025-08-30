#include "libaeth_zkvm.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>
#include <stdbool.h>

// Internal state
static struct {
    bool initialized;
    bool debug_logging;
    char last_error[1024];
    zkvm_log_callback_t log_callback;
    void* log_user_data;
    uint64_t current_memory;
    uint64_t peak_memory;
    uint64_t allocated_memory;
    struct {
        uint64_t proof_cost;
        uint64_t verification_cost;
    } gas_costs[4]; // One for each algorithm
} g_state = {0};

// Default gas costs
static const uint64_t DEFAULT_GAS_COSTS[4][2] = {
    {1000, 100},   // Halo2: proof=1000, verification=100
    {800, 80},     // Noir: proof=800, verification=80
    {1200, 120},   // PLONK: proof=1200, verification=120
    {1500, 150},   // Custom: proof=1500, verification=150
};

// Internal helper functions
static void set_error(const char* format, ...) {
    va_list args;
    va_start(args, format);
    vsnprintf(g_state.last_error, sizeof(g_state.last_error), format, args);
    va_end(args);
    
    if (g_state.debug_logging) {
        printf("ZKVM Error: %s\n", g_state.last_error);
    }
}

static void log_message(const char* format, ...) {
    if (g_state.log_callback) {
        va_list args;
        va_start(args, format);
        char message[1024];
        vsnprintf(message, sizeof(message), format, args);
        va_end(args);
        g_state.log_callback(message, g_state.log_user_data);
    }
}

static void* safe_malloc(size_t size) {
    void* ptr = malloc(size);
    if (ptr) {
        g_state.current_memory += size;
        g_state.allocated_memory += size;
        if (g_state.current_memory > g_state.peak_memory) {
            g_state.peak_memory = g_state.current_memory;
        }
    }
    return ptr;
}

static void safe_free(void* ptr, size_t size) {
    if (ptr) {
        free(ptr);
        g_state.current_memory -= size;
    }
}

static char* safe_strdup(const char* str) {
    if (!str) return NULL;
    size_t len = strlen(str) + 1;
    char* copy = safe_malloc(len);
    if (copy) {
        strcpy(copy, str);
    }
    return copy;
}

static uint8_t* safe_memdup(const uint8_t* data, size_t size) {
    if (!data || size == 0) return NULL;
    uint8_t* copy = safe_malloc(size);
    if (copy) {
        memcpy(copy, data, size);
    }
    return copy;
}

// Public API implementation

int zkvm_init(void) {
    if (g_state.initialized) {
        set_error("ZKVM already initialized");
        return -1;
    }
    
    // Initialize state
    memset(&g_state, 0, sizeof(g_state));
    g_state.initialized = true;
    g_state.debug_logging = false;
    
    // Set default gas costs
    for (int i = 0; i < 4; i++) {
        g_state.gas_costs[i].proof_cost = DEFAULT_GAS_COSTS[i][0];
        g_state.gas_costs[i].verification_cost = DEFAULT_GAS_COSTS[i][1];
    }
    
    log_message("ZKVM initialized successfully");
    return 0;
}

int zkvm_cleanup(void) {
    if (!g_state.initialized) {
        set_error("ZKVM not initialized");
        return -1;
    }
    
    log_message("ZKVM cleanup completed");
    g_state.initialized = false;
    return 0;
}

zk_proof_result_t* zkvm_generate_proof(const zk_proof_request_t* request) {
    if (!g_state.initialized) {
        set_error("ZKVM not initialized");
        return NULL;
    }
    
    if (!request) {
        set_error("Invalid request: NULL pointer");
        return NULL;
    }
    
    if (!request->input || request->input_size == 0) {
        set_error("Invalid request: missing or empty input");
        return NULL;
    }
    
    if (!request->circuit_id) {
        set_error("Invalid request: missing circuit ID");
        return NULL;
    }
    
    log_message("Generating ZK proof for circuit: %s", request->circuit_id);
    
    // Start timing
    clock_t start_time = clock();
    
    // For now, generate a mock proof
    // In a real implementation, this would call the actual ZK proving system
    zk_proof_result_t* result = safe_malloc(sizeof(zk_proof_result_t));
    if (!result) {
        set_error("Failed to allocate memory for result");
        return NULL;
    }
    
    // Create mock proof
    zk_proof_t* proof = safe_malloc(sizeof(zk_proof_t));
    if (!proof) {
        set_error("Failed to allocate memory for proof");
        safe_free(result, sizeof(zk_proof_result_t));
        return NULL;
    }
    
    // Fill proof structure
    proof->algorithm = request->algorithm;
    proof->proof_data = safe_memdup((uint8_t*)"mock_proof_data", 15);
    proof->proof_size = 15;
    proof->public_inputs = safe_memdup(request->input, request->input_size);
    proof->public_inputs_size = request->input_size;
    proof->circuit_hash = safe_memdup((uint8_t*)"mock_circuit_hash", 17);
    proof->circuit_hash_size = 17;
    proof->prover_version = safe_strdup("1.0.0");
    proof->verification_key = safe_memdup((uint8_t*)"mock_verification_key", 22);
    proof->verification_key_size = 22;
    
    // Calculate timing
    clock_t end_time = clock();
    uint64_t generation_time_ms = (uint64_t)((end_time - start_time) * 1000 / CLOCKS_PER_SEC);
    
    // Fill result structure
    result->success = true;
    result->proof = proof;
    result->error_message = NULL;
    result->generation_time_ms = generation_time_ms;
    result->memory_used = proof->proof_size + proof->public_inputs_size + 
                         proof->circuit_hash_size + proof->verification_key_size;
    
    log_message("ZK proof generated successfully in %llu ms", 
                (unsigned long long)generation_time_ms);
    
    return result;
}

zk_verification_result_t* zkvm_verify_proof(const zk_verification_request_t* request) {
    if (!g_state.initialized) {
        set_error("ZKVM not initialized");
        return NULL;
    }
    
    if (!request) {
        set_error("Invalid request: NULL pointer");
        return NULL;
    }
    
    if (!request->proof) {
        set_error("Invalid request: missing proof");
        return NULL;
    }
    
    if (!request->circuit_id) {
        set_error("Invalid request: missing circuit ID");
        return NULL;
    }
    
    log_message("Verifying ZK proof for circuit: %s", request->circuit_id);
    
    // Start timing
    clock_t start_time = clock();
    
    // For now, always return success (mock verification)
    // In a real implementation, this would call the actual ZK verification system
    zk_verification_result_t* result = safe_malloc(sizeof(zk_verification_result_t));
    if (!result) {
        set_error("Failed to allocate memory for result");
        return NULL;
    }
    
    // Calculate timing
    clock_t end_time = clock();
    uint64_t verification_time_ms = (uint64_t)((end_time - start_time) * 1000 / CLOCKS_PER_SEC);
    
    // Fill result structure
    result->success = true;
    result->verification_time_ms = verification_time_ms;
    result->error_message = NULL;
    
    log_message("ZK proof verified successfully in %llu ms", 
                (unsigned long long)verification_time_ms);
    
    return result;
}

int zkvm_register_circuit(const zk_circuit_info_t* circuit) {
    if (!g_state.initialized) {
        set_error("ZKVM not initialized");
        return -1;
    }
    
    if (!circuit) {
        set_error("Invalid circuit: NULL pointer");
        return -1;
    }
    
    if (!circuit->id) {
        set_error("Invalid circuit: missing ID");
        return -1;
    }
    
    log_message("Circuit registration not implemented in mock version");
    return 0; // Success for now
}

zk_circuit_info_t* zkvm_get_circuit(const char* circuit_id) {
    if (!g_state.initialized) {
        set_error("ZKVM not initialized");
        return NULL;
    }
    
    if (!circuit_id) {
        set_error("Invalid circuit ID: NULL pointer");
        return NULL;
    }
    
    log_message("Circuit retrieval not implemented in mock version");
    return NULL;
}

zk_circuit_info_t** zkvm_list_circuits(size_t* count) {
    if (!g_state.initialized) {
        set_error("ZKVM not initialized");
        return NULL;
    }
    
    if (!count) {
        set_error("Invalid count pointer: NULL");
        return NULL;
    }
    
    log_message("Circuit listing not implemented in mock version");
    *count = 0;
    return NULL;
}

zkvm_stats_t zkvm_get_stats(void) {
    zkvm_stats_t stats = {0};
    
    if (g_state.initialized) {
        stats.total_circuits = 0;      // Mock implementation
        stats.total_provers = 1;       // Mock prover
        stats.total_verifiers = 1;     // Mock verifier
        stats.supported_algorithms = 4; // Halo2, Noir, PLONK, Custom
    }
    
    return stats;
}

void zkvm_free_proof(zk_proof_t* proof) {
    if (!proof) return;
    
    safe_free(proof->proof_data, proof->proof_size);
    safe_free(proof->public_inputs, proof->public_inputs_size);
    safe_free(proof->circuit_hash, proof->circuit_hash_size);
    safe_free(proof->prover_version, strlen(proof->prover_version) + 1);
    safe_free(proof->verification_key, proof->verification_key_size);
    safe_free(proof, sizeof(zk_proof_t));
}

void zkvm_free_proof_result(zk_proof_result_t* result) {
    if (!result) return;
    
    if (result->proof) {
        zkvm_free_proof(result->proof);
    }
    
    if (result->error_message) {
        safe_free(result->error_message, strlen(result->error_message) + 1);
    }
    
    safe_free(result, sizeof(zk_proof_result_t));
}

void zkvm_free_verification_result(zk_verification_result_t* result) {
    if (!result) return;
    
    if (result->error_message) {
        safe_free(result->error_message, strlen(result->error_message) + 1);
    }
    
    safe_free(result, sizeof(zk_verification_result_t));
}

void zkvm_free_circuit_info(zk_circuit_info_t* circuit) {
    if (!circuit) return;
    
    safe_free(circuit->id, strlen(circuit->id) + 1);
    safe_free(circuit->name, strlen(circuit->name) + 1);
    if (circuit->description) {
        safe_free(circuit->description, strlen(circuit->description) + 1);
    }
    safe_free(circuit->circuit_hash, circuit->circuit_hash_size);
    safe_free(circuit->verification_key, circuit->verification_key_size);
    safe_free(circuit, sizeof(zk_circuit_info_t));
}

void zkvm_free_circuit_list(zk_circuit_info_t** circuits, size_t count) {
    if (!circuits) return;
    
    for (size_t i = 0; i < count; i++) {
        if (circuits[i]) {
            zkvm_free_circuit_info(circuits[i]);
        }
    }
    
    safe_free(circuits, count * sizeof(zk_circuit_info_t*));
}

const char* zkvm_get_last_error(void) {
    return g_state.last_error;
}

void zkvm_set_error(const char* message) {
    if (message) {
        strncpy(g_state.last_error, message, sizeof(g_state.last_error) - 1);
        g_state.last_error[sizeof(g_state.last_error) - 1] = '\0';
    }
}

bool zkvm_is_algorithm_supported(zk_algorithm_t algorithm) {
    return algorithm >= ZK_ALGORITHM_HALO2 && algorithm <= ZK_ALGORITHM_CUSTOM;
}

const char* zkvm_algorithm_name(zk_algorithm_t algorithm) {
    switch (algorithm) {
        case ZK_ALGORITHM_HALO2: return "halo2";
        case ZK_ALGORITHM_NOIR: return "noir";
        case ZK_ALGORITHM_PLONK: return "plonk";
        case ZK_ALGORITHM_CUSTOM: return "custom";
        default: return "unknown";
    }
}

zk_algorithm_t zkvm_parse_algorithm(const char* name) {
    if (!name) return ZK_ALGORITHM_CUSTOM;
    
    if (strcmp(name, "halo2") == 0) return ZK_ALGORITHM_HALO2;
    if (strcmp(name, "noir") == 0) return ZK_ALGORITHM_NOIR;
    if (strcmp(name, "plonk") == 0) return ZK_ALGORITHM_PLONK;
    if (strcmp(name, "custom") == 0) return ZK_ALGORITHM_CUSTOM;
    
    return ZK_ALGORITHM_CUSTOM;
}

uint64_t zkvm_get_default_gas_cost(zk_algorithm_t algorithm) {
    if (algorithm >= 0 && algorithm < 4) {
        return DEFAULT_GAS_COSTS[algorithm][0];
    }
    return 1000; // Default fallback
}

uint64_t zkvm_get_default_verification_gas_cost(zk_algorithm_t algorithm) {
    if (algorithm >= 0 && algorithm < 4) {
        return DEFAULT_GAS_COSTS[algorithm][1];
    }
    return 100; // Default fallback
}

int zkvm_set_custom_gas_costs(zk_algorithm_t algorithm, uint64_t proof_cost, uint64_t verification_cost) {
    if (!g_state.initialized) {
        set_error("ZKVM not initialized");
        return -1;
    }
    
    if (algorithm < 0 || algorithm >= 4) {
        set_error("Invalid algorithm");
        return -1;
    }
    
    g_state.gas_costs[algorithm].proof_cost = proof_cost;
    g_state.gas_costs[algorithm].verification_cost = verification_cost;
    
    log_message("Custom gas costs set for algorithm %s: proof=%llu, verification=%llu",
                zkvm_algorithm_name(algorithm),
                (unsigned long long)proof_cost,
                (unsigned long long)verification_cost);
    
    return 0;
}

void zkvm_get_memory_stats(uint64_t* current_memory, uint64_t* peak_memory, uint64_t* allocated_memory) {
    if (current_memory) *current_memory = g_state.current_memory;
    if (peak_memory) *peak_memory = g_state.peak_memory;
    if (allocated_memory) *allocated_memory = g_state.allocated_memory;
}

void zkvm_reset_memory_stats(void) {
    g_state.current_memory = 0;
    g_state.peak_memory = 0;
    g_state.allocated_memory = 0;
    log_message("Memory statistics reset");
}

void zkvm_set_debug_logging(bool enabled) {
    g_state.debug_logging = enabled;
    log_message("Debug logging %s", enabled ? "enabled" : "disabled");
}

void zkvm_set_log_callback(zkvm_log_callback_t callback, void* user_data) {
    g_state.log_callback = callback;
    g_state.log_user_data = user_data;
}

const char* zkvm_get_version(void) {
    return "1.0.0-mock";
}

char** zkvm_get_supported_algorithms(size_t* count) {
    if (!count) return NULL;
    
    *count = 4;
    char** algorithms = safe_malloc(4 * sizeof(char*));
    if (!algorithms) return NULL;
    
    algorithms[0] = safe_strdup("halo2");
    algorithms[1] = safe_strdup("noir");
    algorithms[2] = safe_strdup("plonk");
    algorithms[3] = safe_strdup("custom");
    
    return algorithms;
}

void zkvm_free_string_array(char** strings, size_t count) {
    if (!strings) return;
    
    for (size_t i = 0; i < count; i++) {
        if (strings[i]) {
            safe_free(strings[i], strlen(strings[i]) + 1);
        }
    }
    
    safe_free(strings, count * sizeof(char*));
}
