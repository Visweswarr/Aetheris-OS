#include "../libaeth_time.h"
#include <stdio.h>
#include <assert.h>
#include <unistd.h>
#include <string.h>

#define TEST_ITERATIONS 1000
#define CALIBRATION_TARGET_US 1000

static void test_initialization(void) {
    printf("Testing timing system initialization...\n");
    
    assert(!aeth_time_ready());
    
    bool init_result = aeth_time_init();
    assert(init_result);
    
    assert(aeth_time_ready());
    printf("✓ Initialization successful\n");
}

static void test_monotonic_behavior(void) {
    printf("Testing monotonic behavior...\n");
    
    uint64_t prev = aeth_ticks_now();
    assert(prev > 0);
    
    for (int i = 0; i < TEST_ITERATIONS; i++) {
        uint64_t current = aeth_ticks_now();
        assert(current >= prev);
        prev = current;
    }
    
    printf("✓ Monotonic behavior verified (%d iterations)\n", TEST_ITERATIONS);
}

static void test_calibration_accuracy(void) {
    printf("Testing calibration accuracy...\n");
    
    uint64_t start = aeth_ticks_now();
    usleep(CALIBRATION_TARGET_US);
    uint64_t end = aeth_ticks_now();
    
    uint64_t measured_us = end - start;
    double error = (double)abs((int64_t)measured_us - CALIBRATION_TARGET_US) / CALIBRATION_TARGET_US;
    
    printf("Measured: %lu μs, Expected: %d μs, Error: %.2f%%\n", 
           measured_us, CALIBRATION_TARGET_US, error * 100.0);
    
    assert(error < 0.05);
    printf("✓ Calibration accuracy verified (error < 5%%)\n");
}

static void test_conversion_consistency(void) {
    printf("Testing conversion consistency...\n");
    
    uint64_t start = aeth_ticks_now();
    usleep(1000);
    uint64_t end = aeth_ticks_now();
    
    uint64_t tick_diff = end - start;
    double us_diff = aeth_ticks_to_us(tick_diff);
    
    printf("Tick difference: %lu, Converted: %.2f μs\n", tick_diff, us_diff);
    
    assert(us_diff > 0);
    assert(us_diff < 10000);
    
    printf("✓ Conversion consistency verified\n");
}

static void test_resolution(void) {
    printf("Testing timing resolution...\n");
    
    uint64_t resolution = aeth_time_resolution_ns();
    printf("Resolution: %lu ns\n", resolution);
    
    assert(resolution > 0);
    assert(resolution < 1000000);
    
    printf("✓ Resolution verified\n");
}

static void test_calibration_process(void) {
    printf("Testing calibration process...\n");
    
    uint64_t total_calls_before, calibration_count_before, last_calibration_before;
    aeth_time_stats(&total_calls_before, &calibration_count_before, &last_calibration_before);
    
    bool cal_result = aeth_time_calibrate();
    assert(cal_result);
    
    uint64_t total_calls_after, calibration_count_after, last_calibration_after;
    aeth_time_stats(&total_calls_after, &calibration_count_after, &last_calibration_after);
    
    assert(calibration_count_after > calibration_count_before);
    assert(last_calibration_after > last_calibration_before);
    
    printf("✓ Calibration process verified\n");
    printf("  Calibrations: %lu -> %lu\n", calibration_count_before, calibration_count_after);
}

static void test_stats_tracking(void) {
    printf("Testing statistics tracking...\n");
    
    uint64_t total_calls, calibration_count, last_calibration;
    aeth_time_stats(&total_calls, &calibration_count, &last_calibration);
    
    printf("Stats: calls=%lu, calibrations=%lu, last_cal=%lu\n", 
           total_calls, calibration_count, last_calibration);
    
    assert(total_calls > 0);
    assert(calibration_count > 0);
    assert(last_calibration > 0);
    
    printf("✓ Statistics tracking verified\n");
}

static void test_repeated_calls(void) {
    printf("Testing repeated timing calls...\n");
    
    uint64_t start = aeth_ticks_now();
    
    for (int i = 0; i < 100; i++) {
        uint64_t current = aeth_ticks_now();
        assert(current >= start);
    }
    
    uint64_t end = aeth_ticks_now();
    assert(end > start);
    
    printf("✓ Repeated calls verified\n");
}

static void test_edge_cases(void) {
    printf("Testing edge cases...\n");
    
    uint64_t zero_diff = aeth_ticks_to_us(0);
    assert(zero_diff == 0.0);
    
    uint64_t small_diff = aeth_ticks_to_us(1);
    assert(small_diff >= 0.0);
    
    printf("✓ Edge cases handled correctly\n");
}

static void test_concurrent_access(void) {
    printf("Testing concurrent access simulation...\n");
    
    uint64_t start = aeth_ticks_now();
    
    for (int i = 0; i < 50; i++) {
        uint64_t t1 = aeth_ticks_now();
        uint64_t t2 = aeth_ticks_now();
        uint64_t t3 = aeth_ticks_now();
        
        assert(t3 >= t2);
        assert(t2 >= t1);
        assert(t1 >= start);
    }
    
    printf("✓ Concurrent access simulation verified\n");
}

int main(void) {
    printf("=== NGFS Timing Library Tests ===\n\n");
    
    test_initialization();
    test_monotonic_behavior();
    test_calibration_accuracy();
    test_conversion_consistency();
    test_resolution();
    test_calibration_process();
    test_stats_tracking();
    test_repeated_calls();
    test_edge_cases();
    test_concurrent_access();
    
    printf("\n=== All Tests Passed! ===\n");
    
    uint64_t total_calls, calibration_count, last_calibration;
    aeth_time_stats(&total_calls, &calibration_count, &last_calibration);
    
    printf("\nFinal Statistics:\n");
    printf("  Total timing calls: %lu\n", total_calls);
    printf("  Calibrations performed: %lu\n", calibration_count);
    printf("  Last calibration: %lu\n", last_calibration);
    printf("  Timing resolution: %lu ns\n", aeth_time_resolution_ns());
    
    return 0;
}
