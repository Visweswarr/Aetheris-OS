//! C TLS Integration Tests for Aetheris OS
//! 
//! This file contains comprehensive tests for the C TLS bindings.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <time.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#include "aetheris/pqc_tls_v1.h"

// Test helper functions
static void test_assert(int condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "TEST FAILED: %s\n", message);
        exit(1);
    }
    printf("✓ %s\n", message);
}

static void test_tls_manager_initialization() {
    printf("Testing TLS manager initialization...\n");
    
    pqc_tls_manager_t manager;
    int result = pqc_tls_manager_init(&manager);
    test_assert(result == PQC_TLS_SUCCESS, "TLS manager initialization");
    
    result = pqc_tls_manager_cleanup(&manager);
    test_assert(result == PQC_TLS_SUCCESS, "TLS manager cleanup");
}

static void test_pqc_key_generation() {
    printf("Testing PQC key generation...\n");
    
    pqc_tls_manager_t manager;
    pqc_tls_manager_init(&manager);
    
    // Test Kyber key generation
    pqc_key_pair_t kyber_keys;
    int result = pqc_tls_generate_key_pair(&manager, PQC_ALGORITHM_KYBER512, 
                                          PQC_KEY_USAGE_KEY_ENCAPSULATION, &kyber_keys);
    test_assert(result == PQC_TLS_SUCCESS, "Kyber512 key generation");
    test_assert(kyber_keys.public_key_len > 0, "Kyber512 public key length");
    test_assert(strlen(kyber_keys.id) > 0, "Kyber512 key ID");
    
    // Test Dilithium key generation
    pqc_key_pair_t dilithium_keys;
    result = pqc_tls_generate_key_pair(&manager, PQC_ALGORITHM_DILITHIUM2, 
                                      PQC_KEY_USAGE_DIGITAL_SIGNATURE, &dilithium_keys);
    test_assert(result == PQC_TLS_SUCCESS, "Dilithium2 key generation");
    test_assert(dilithium_keys.public_key_len > 0, "Dilithium2 public key length");
    test_assert(strlen(dilithium_keys.id) > 0, "Dilithium2 key ID");
    
    // Cleanup
    pqc_tls_free_key_pair(&kyber_keys);
    pqc_tls_free_key_pair(&dilithium_keys);
    pqc_tls_manager_cleanup(&manager);
}

static void test_did_certificate_creation() {
    printf("Testing DID certificate creation...\n");
    
    pqc_tls_manager_t manager;
    pqc_tls_manager_init(&manager);
    
    // Generate keys
    pqc_key_pair_t kyber_keys, dilithium_keys;
    pqc_tls_generate_key_pair(&manager, PQC_ALGORITHM_KYBER512, 
                              PQC_KEY_USAGE_KEY_ENCAPSULATION, &kyber_keys);
    pqc_tls_generate_key_pair(&manager, PQC_ALGORITHM_DILITHIUM2, 
                              PQC_KEY_USAGE_DIGITAL_SIGNATURE, &dilithium_keys);
    
    // Create certificate
    did_certificate_t certificate;
    int result = pqc_tls_create_did_certificate(&manager, "did:aetheris:test:client",
                                               "Test Client", 365, &kyber_keys, &certificate);
    test_assert(result == PQC_TLS_SUCCESS, "DID certificate creation");
    test_assert(strcmp(certificate.did, "did:aetheris:test:client") == 0, "Certificate DID");
    test_assert(certificate.certificate_len > 0, "Certificate data length");
    test_assert(certificate.pqc_signature_len > 0, "PQC signature length");
    test_assert(!certificate.revoked, "Certificate not revoked");
    
    // Cleanup
    pqc_tls_free_did_certificate(&certificate);
    pqc_tls_free_key_pair(&kyber_keys);
    pqc_tls_free_key_pair(&dilithium_keys);
    pqc_tls_manager_cleanup(&manager);
}

static void test_tls_session_creation() {
    printf("Testing TLS session creation...\n");
    
    pqc_tls_manager_t manager;
    pqc_tls_manager_init(&manager);
    
    tls_session_t session;
    int result = pqc_tls_create_session(&manager, "test_session", &session);
    test_assert(result == PQC_TLS_SUCCESS, "TLS session creation");
    test_assert(strcmp(session.session_id, "test_session") == 0, "Session ID");
    test_assert(session.state == 0, "Session initial state");
    test_assert(session.bytes_sent == 0, "Initial bytes sent");
    test_assert(session.bytes_received == 0, "Initial bytes received");
    
    pqc_tls_manager_cleanup(&manager);
}

static void test_tls_handshake() {
    printf("Testing TLS handshake...\n");
    
    pqc_tls_manager_t manager;
    pqc_tls_manager_init(&manager);
    
    tls_session_t session;
    pqc_tls_create_session(&manager, "handshake_test", &session);
    
    pqc_tls_handshake_result_t result;
    int ret = pqc_tls_mtls_handshake(&manager, "handshake_test", 
                                    "did:aetheris:test:client", 
                                    "did:aetheris:test:server", &result);
    test_assert(ret == PQC_TLS_SUCCESS, "mTLS handshake execution");
    test_assert(result.success, "Handshake success");
    test_assert(result.handshake_time_ms > 0, "Handshake time > 0");
    test_assert(result.cipher_suite == CIPHER_SUITE_HYBRID_AES_KYBER, "Cipher suite");
    test_assert(result.tls_version == TLS_VERSION_1_3, "TLS version");
    
    pqc_tls_manager_cleanup(&manager);
}

static void test_tls_encryption_decryption() {
    printf("Testing TLS encryption/decryption...\n");
    
    pqc_tls_manager_t manager;
    pqc_tls_manager_init(&manager);
    
    tls_session_t session;
    pqc_tls_create_session(&manager, "encryption_test", &session);
    
    // Test data
    const char *original_data = "Hello, Post-Quantum TLS!";
    size_t data_len = strlen(original_data);
    
    // Encrypt data
    uint8_t encrypted_data[1024];
    size_t encrypted_len = sizeof(encrypted_data);
    int result = pqc_tls_encrypt_data(&manager, "encryption_test", 
                                     (const uint8_t *)original_data, data_len,
                                     encrypted_data, &encrypted_len);
    test_assert(result == PQC_TLS_SUCCESS, "Data encryption");
    test_assert(encrypted_len > 0, "Encrypted data length > 0");
    
    // Decrypt data
    uint8_t decrypted_data[1024];
    size_t decrypted_len = sizeof(decrypted_data);
    result = pqc_tls_decrypt_data(&manager, "encryption_test", 
                                 encrypted_data, encrypted_len,
                                 decrypted_data, &decrypted_len);
    test_assert(result == PQC_TLS_SUCCESS, "Data decryption");
    test_assert(decrypted_len == data_len, "Decrypted data length matches original");
    test_assert(memcmp(decrypted_data, original_data, data_len) == 0, "Decrypted data matches original");
    
    pqc_tls_manager_cleanup(&manager);
}

static void test_tls_rekeying() {
    printf("Testing TLS rekeying...\n");
    
    pqc_tls_manager_t manager;
    pqc_tls_manager_init(&manager);
    
    tls_session_t session;
    pqc_tls_create_session(&manager, "rekey_test", &session);
    
    // Perform rekeying
    int result = pqc_tls_rekey_session(&manager, "rekey_test", PQC_ALGORITHM_KYBER768);
    test_assert(result == PQC_TLS_SUCCESS, "Session rekeying");
    
    // Verify session is still functional after rekeying
    const char *test_data = "Test data after rekeying";
    size_t data_len = strlen(test_data);
    
    uint8_t encrypted_data[1024];
    size_t encrypted_len = sizeof(encrypted_data);
    result = pqc_tls_encrypt_data(&manager, "rekey_test", 
                                 (const uint8_t *)test_data, data_len,
                                 encrypted_data, &encrypted_len);
    test_assert(result == PQC_TLS_SUCCESS, "Encryption after rekeying");
    
    uint8_t decrypted_data[1024];
    size_t decrypted_len = sizeof(decrypted_data);
    result = pqc_tls_decrypt_data(&manager, "rekey_test", 
                                 encrypted_data, encrypted_len,
                                 decrypted_data, &decrypted_len);
    test_assert(result == PQC_TLS_SUCCESS, "Decryption after rekeying");
    test_assert(memcmp(decrypted_data, test_data, data_len) == 0, "Data integrity after rekeying");
    
    pqc_tls_manager_cleanup(&manager);
}

static void test_tls_signature_verification() {
    printf("Testing TLS signature verification...\n");
    
    pqc_tls_manager_t manager;
    pqc_tls_manager_init(&manager);
    
    // Generate key pair for signing
    pqc_key_pair_t key_pair;
    pqc_tls_generate_key_pair(&manager, PQC_ALGORITHM_DILITHIUM2, 
                              PQC_KEY_USAGE_DIGITAL_SIGNATURE, &key_pair);
    
    // Test data
    const char *test_data = "Test data for signature";
    size_t data_len = strlen(test_data);
    
    // Sign data
    uint8_t signature[1024];
    size_t signature_len = sizeof(signature);
    int result = pqc_tls_sign_data(&manager, (const uint8_t *)test_data, data_len,
                                  key_pair.private_key_id, signature, &signature_len,
                                  PQC_ALGORITHM_DILITHIUM2);
    test_assert(result == PQC_TLS_SUCCESS, "Data signing");
    test_assert(signature_len > 0, "Signature length > 0");
    
    // Verify signature
    bool is_valid;
    result = pqc_tls_verify_signature(&manager, (const uint8_t *)test_data, data_len,
                                     signature, signature_len,
                                     key_pair.public_key, key_pair.public_key_len,
                                     PQC_ALGORITHM_DILITHIUM2, &is_valid);
    test_assert(result == PQC_TLS_SUCCESS, "Signature verification");
    test_assert(is_valid, "Signature is valid");
    
    // Cleanup
    pqc_tls_free_key_pair(&key_pair);
    pqc_tls_manager_cleanup(&manager);
}

static void test_tls_manager_statistics() {
    printf("Testing TLS manager statistics...\n");
    
    pqc_tls_manager_t manager;
    pqc_tls_manager_init(&manager);
    
    // Create some sessions and perform operations
    tls_session_t session1, session2;
    pqc_tls_create_session(&manager, "stats_test_1", &session1);
    pqc_tls_create_session(&manager, "stats_test_2", &session2);
    
    // Perform handshakes
    pqc_tls_handshake_result_t result;
    pqc_tls_mtls_handshake(&manager, "stats_test_1", "client1", "server1", &result);
    pqc_tls_mtls_handshake(&manager, "stats_test_2", "client2", "server2", &result);
    
    // Get statistics
    pqc_tls_manager_stats_t stats;
    int ret = pqc_tls_get_manager_stats(&manager, &stats);
    test_assert(ret == PQC_TLS_SUCCESS, "Statistics retrieval");
    test_assert(stats.active_sessions >= 2, "Active sessions count");
    test_assert(stats.total_handshakes >= 2, "Total handshakes count");
    
    pqc_tls_manager_cleanup(&manager);
}

static void test_tls_error_handling() {
    printf("Testing TLS error handling...\n");
    
    pqc_tls_manager_t manager;
    pqc_tls_manager_init(&manager);
    
    // Test invalid parameters
    int result = pqc_tls_generate_key_pair(NULL, PQC_ALGORITHM_KYBER512, 
                                          PQC_KEY_USAGE_KEY_ENCAPSULATION, NULL);
    test_assert(result == PQC_TLS_ERROR_INVALID_PARAMETER, "Invalid parameter handling");
    
    // Test unsupported algorithm
    pqc_key_pair_t key_pair;
    result = pqc_tls_generate_key_pair(&manager, 999, PQC_KEY_USAGE_KEY_ENCAPSULATION, &key_pair);
    test_assert(result == PQC_TLS_ERROR_UNSUPPORTED_ALGORITHM, "Unsupported algorithm handling");
    
    // Test buffer too small
    uint8_t small_buffer[1];
    size_t buffer_len = 1;
    result = pqc_tls_encrypt_data(&manager, "nonexistent_session", 
                                 (const uint8_t *)"test", 4,
                                 small_buffer, &buffer_len);
    test_assert(result == PQC_TLS_ERROR_BUFFER_TOO_SMALL, "Buffer too small handling");
    
    pqc_tls_manager_cleanup(&manager);
}

static void test_tls_performance() {
    printf("Testing TLS performance...\n");
    
    pqc_tls_manager_t manager;
    pqc_tls_manager_init(&manager);
    
    tls_session_t session;
    pqc_tls_create_session(&manager, "perf_test", &session);
    
    // Performance test data
    const size_t data_size = 1024; // 1KB
    uint8_t *test_data = malloc(data_size);
    memset(test_data, 0xAA, data_size);
    
    const int iterations = 1000;
    clock_t start = clock();
    
    for (int i = 0; i < iterations; i++) {
        uint8_t encrypted_data[2048];
        size_t encrypted_len = sizeof(encrypted_data);
        pqc_tls_encrypt_data(&manager, "perf_test", test_data, data_size,
                            encrypted_data, &encrypted_len);
        
        uint8_t decrypted_data[2048];
        size_t decrypted_len = sizeof(decrypted_data);
        pqc_tls_decrypt_data(&manager, "perf_test", encrypted_data, encrypted_len,
                            decrypted_data, &decrypted_len);
    }
    
    clock_t end = clock();
    double cpu_time = ((double)(end - start)) / CLOCKS_PER_SEC;
    double throughput = (data_size * iterations * 2) / cpu_time; // *2 for encrypt+decrypt
    
    printf("TLS Performance: %.2f MB/s\n", throughput / 1_000_000.0);
    test_assert(throughput > 100_000_000.0, "Performance > 100 MB/s"); // > 100 MB/s
    
    free(test_data);
    pqc_tls_manager_cleanup(&manager);
}

static void test_tls_socket_integration() {
    printf("Testing TLS socket integration...\n");
    
    // Test TLS-enabled socket creation
    int sockfd = tls_socket(AF_INET, SOCK_STREAM, 0, "did:aetheris:test:client", "did:aetheris:test:server");
    test_assert(sockfd >= 0, "TLS socket creation");
    
    // Test TLS handshake
    int result = tls_handshake(sockfd);
    test_assert(result == 0, "TLS handshake");
    
    // Test TLS send/recv
    const char *test_message = "Hello TLS!";
    ssize_t sent = tls_send(sockfd, test_message, strlen(test_message), 0);
    test_assert(sent > 0, "TLS send");
    
    char recv_buffer[1024];
    ssize_t received = tls_recv(sockfd, recv_buffer, sizeof(recv_buffer), 0);
    test_assert(received > 0, "TLS recv");
    
    // Test TLS statistics
    pqc_tls_manager_stats_t stats;
    result = tls_get_stats(sockfd, &stats);
    test_assert(result == 0, "TLS statistics retrieval");
    
    // Test TLS rekeying
    result = tls_rekey(sockfd, PQC_ALGORITHM_KYBER768);
    test_assert(result == 0, "TLS rekeying");
    
    close(sockfd);
}

int main() {
    printf("Running C TLS Integration Tests for Aetheris OS\n");
    printf("===============================================\n\n");
    
    test_tls_manager_initialization();
    test_pqc_key_generation();
    test_did_certificate_creation();
    test_tls_session_creation();
    test_tls_handshake();
    test_tls_encryption_decryption();
    test_tls_rekeying();
    test_tls_signature_verification();
    test_tls_manager_statistics();
    test_tls_error_handling();
    test_tls_performance();
    test_tls_socket_integration();
    
    printf("\n===============================================\n");
    printf("All C TLS tests passed successfully!\n");
    return 0;
}
