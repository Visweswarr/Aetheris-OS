#ifndef LIBAETH_ZKVM_H
#define LIBAETH_ZKVM_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @file libaeth_zkvm.h
 * @brief NGFS ZKVM adapter for Zero-Knowledge proof generation and verification
 * 
 * This library provides a thin wrapper around ZK proving systems (Halo2, Noir, PLONK)
 * for generating and verifying proofs of smart contract execution.
 */

/**
 * ZK algorithm types
 */
typedef enum {
    ZK_ALGORITHM_HALO2 = 0,    ///< Halo2 proving system
    ZK_ALGORITHM_NOIR = 1,     ///< Noir proving system
    ZK_ALGORITHM_PLONK = 2,    ///< PLONK proving system
    ZK_ALGORITHM_CUSTOM = 255  ///< Custom proving system
} zk_algorithm_t;

/**
 * ZK proof structure
 */
typedef struct {
    zk_algorithm_t algorithm;   ///< Algorithm used for proof generation
    uint8_t* proof_data;        ///< Serialized proof data
    size_t proof_size;          ///< Size of proof data in bytes
    uint8_t* public_inputs;     ///< Public inputs for verification
    size_t public_inputs_size;  ///< Size of public inputs in bytes
    uint8_t* circuit_hash;      ///< Hash of the circuit
    size_t circuit_hash_size;   ///< Size of circuit hash in bytes
    char* prover_version;       ///< Version of the prover
    uint8_t* verification_key;  ///< Verification key for the circuit
    size_t verification_key_size; ///< Size of verification key in bytes
} zk_proof_t;

/**
 * ZK proof generation request
 */
typedef struct {
    uint8_t* input;             ///< Input data for proof generation
    size_t input_size;          ///< Size of input data in bytes
    zk_algorithm_t algorithm;   ///< ZK algorithm to use
    char* circuit_id;           ///< Circuit identifier
    uint8_t* parameters;        ///< Additional parameters
    size_t parameters_size;     ///< Size of parameters in bytes
} zk_proof_request_t;

/**
 * ZK proof verification request
 */
typedef struct {
    zk_proof_t* proof;          ///< Proof to verify
    uint8_t* public_inputs;     ///< Public inputs for verification
    size_t public_inputs_size;  ///< Size of public inputs in bytes
    char* circuit_id;           ///< Circuit identifier
} zk_verification_request_t;

/**
 * ZK proof generation result
 */
typedef struct {
    bool success;                ///< Whether generation succeeded
    zk_proof_t* proof;          ///< Generated proof (if successful)
    char* error_message;        ///< Error message (if failed)
    uint64_t generation_time_ms; ///< Generation time in milliseconds
    uint64_t memory_used;       ///< Memory used during generation
} zk_proof_result_t;

/**
 * ZK proof verification result
 */
typedef struct {
    bool success;                ///< Whether verification succeeded
    uint64_t verification_time_ms; ///< Verification time in milliseconds
    char* error_message;        ///< Error message (if failed)
} zk_verification_result_t;

/**
 * Circuit information
 */
typedef struct {
    char* id;                   ///< Circuit identifier
    char* name;                 ///< Circuit name
    char* description;          ///< Circuit description
    zk_algorithm_t algorithm;   ///< ZK algorithm used
    uint8_t* circuit_hash;      ///< Circuit hash
    size_t circuit_hash_size;   ///< Size of circuit hash in bytes
    uint8_t* verification_key;  ///< Verification key
    size_t verification_key_size; ///< Size of verification key in bytes
    uint64_t circuit_size;      ///< Circuit size (gates)
    uint64_t max_proof_size;    ///< Maximum proof size
    uint64_t estimated_proving_time_ms; ///< Estimated proving time
    uint64_t estimated_verification_time_ms; ///< Estimated verification time
} zk_circuit_info_t;

/**
 * ZKVM service statistics
 */
typedef struct {
    uint64_t total_circuits;    ///< Total number of circuits
    uint64_t total_provers;     ///< Total number of provers
    uint64_t total_verifiers;   ///< Total number of verifiers
    uint64_t supported_algorithms; ///< Number of supported algorithms
} zkvm_stats_t;

/**
 * Initialize the ZKVM service
 * 
 * @return 0 on success, negative value on failure
 */
int zkvm_init(void);

/**
 * Cleanup the ZKVM service
 * 
 * @return 0 on success, negative value on failure
 */
int zkvm_cleanup(void);

/**
 * Generate a ZK proof
 * 
 * @param request Proof generation request
 * @return Proof generation result (caller must free with zkvm_free_proof_result)
 */
zk_proof_result_t* zkvm_generate_proof(const zk_proof_request_t* request);

/**
 * Verify a ZK proof
 * 
 * @param request Proof verification request
 * @return Proof verification result (caller must free with zkvm_free_verification_result)
 */
zk_verification_result_t* zkvm_verify_proof(const zk_verification_request_t* request);

/**
 * Register a circuit with the ZKVM service
 * 
 * @param circuit Circuit information
 * @return 0 on success, negative value on failure
 */
int zkvm_register_circuit(const zk_circuit_info_t* circuit);

/**
 * Get circuit information by ID
 * 
 * @param circuit_id Circuit identifier
 * @return Circuit information (caller must free with zkvm_free_circuit_info)
 */
zk_circuit_info_t* zkvm_get_circuit(const char* circuit_id);

/**
 * List all available circuits
 * 
 * @param count Pointer to store the number of circuits
 * @return Array of circuit information (caller must free with zkvm_free_circuit_list)
 */
zk_circuit_info_t** zkvm_list_circuits(size_t* count);

/**
 * Get ZKVM service statistics
 * 
 * @return Service statistics
 */
zkvm_stats_t zkvm_get_stats(void);

/**
 * Free a proof structure
 * 
 * @param proof Proof to free
 */
void zkvm_free_proof(zk_proof_t* proof);

/**
 * Free a proof generation result
 * 
 * @param result Result to free
 */
void zkvm_free_proof_result(zk_proof_result_t* result);

/**
 * Free a proof verification result
 * 
 * @param result Result to free
 */
void zkvm_free_verification_result(zk_verification_result_t* result);

/**
 * Free a circuit information structure
 * 
 * @param circuit Circuit to free
 */
void zkvm_free_circuit_info(zk_circuit_info_t* circuit);

/**
 * Free a list of circuits
 * 
 * @param circuits Array of circuits to free
 * @param count Number of circuits in the array
 */
void zkvm_free_circuit_list(zk_circuit_info_t** circuits, size_t count);

/**
 * Get the last error message
 * 
 * @return Last error message (caller must not free)
 */
const char* zkvm_get_last_error(void);

/**
 * Set error message
 * 
 * @param message Error message
 */
void zkvm_set_error(const char* message);

/**
 * Check if an algorithm is supported
 * 
 * @param algorithm Algorithm to check
 * @return true if supported, false otherwise
 */
bool zkvm_is_algorithm_supported(zk_algorithm_t algorithm);

/**
 * Get algorithm name as string
 * 
 * @param algorithm Algorithm type
 * @return Algorithm name string
 */
const char* zkvm_algorithm_name(zk_algorithm_t algorithm);

/**
 * Parse algorithm name to type
 * 
 * @param name Algorithm name string
 * @return Algorithm type, or ZK_ALGORITHM_CUSTOM if not recognized
 */
zk_algorithm_t zkvm_parse_algorithm(const char* name);

/**
 * Get default gas cost for ZK proof generation
 * 
 * @param algorithm Algorithm to get cost for
 * @return Gas cost in units
 */
uint64_t zkvm_get_default_gas_cost(zk_algorithm_t algorithm);

/**
 * Get default gas cost for ZK proof verification
 * 
 * @param algorithm Algorithm to get cost for
 * @return Gas cost in units
 */
uint64_t zkvm_get_default_verification_gas_cost(zk_algorithm_t algorithm);

/**
 * Set custom gas costs for an algorithm
 * 
 * @param algorithm Algorithm to set costs for
 * @param proof_cost Gas cost for proof generation
 * @param verification_cost Gas cost for proof verification
 * @return 0 on success, negative value on failure
 */
int zkvm_set_custom_gas_costs(zk_algorithm_t algorithm, uint64_t proof_cost, uint64_t verification_cost);

/**
 * Get memory usage statistics
 * 
 * @param current_memory Current memory usage in bytes
 * @param peak_memory Peak memory usage in bytes
 * @param allocated_memory Total allocated memory in bytes
 */
void zkvm_get_memory_stats(uint64_t* current_memory, uint64_t* peak_memory, uint64_t* allocated_memory);

/**
 * Reset memory statistics
 */
void zkvm_reset_memory_stats(void);

/**
 * Enable or disable debug logging
 * 
 * @param enabled true to enable, false to disable
 */
void zkvm_set_debug_logging(bool enabled);

/**
 * Set log callback function
 * 
 * @param callback Function to call for logging (NULL to disable)
 */
typedef void (*zkvm_log_callback_t)(const char* message, void* user_data);
void zkvm_set_log_callback(zkvm_log_callback_t callback, void* user_data);

/**
 * Get library version
 * 
 * @return Version string
 */
const char* zkvm_get_version(void);

/**
 * Get supported algorithms as string array
 * 
 * @param count Pointer to store the number of algorithms
 * @return Array of algorithm names (caller must free with zkvm_free_string_array)
 */
char** zkvm_get_supported_algorithms(size_t* count);

/**
 * Free a string array
 * 
 * @param strings Array of strings to free
 * @param count Number of strings in the array
 */
void zkvm_free_string_array(char** strings, size_t count);

#ifdef __cplusplus
}
#endif

#endif // LIBAETH_ZKVM_H
