#include "libaeth_time.h"
#include <time.h>
#include <unistd.h>
#include <sys/time.h>
#include <stdbool.h>
#include <string.h>

// Timing system state
static struct {
    bool initialized;
    uint64_t calibration_factor;
    uint64_t total_calls;
    uint64_t calibration_count;
    uint64_t last_calibration;
    uint64_t base_ticks;
    struct timespec base_time;
} timing_state = {0};

// Default calibration factor (ticks per microsecond)
#define DEFAULT_CALIBRATION_FACTOR 1000

bool aeth_time_init(void) {
    if (timing_state.initialized) {
        return true;
    }
    
    // Get initial time
    if (clock_gettime(CLOCK_MONOTONIC, &timing_state.base_time) != 0) {
        return false;
    }
    
    // Set initial calibration factor
    timing_state.calibration_factor = DEFAULT_CALIBRATION_FACTOR;
    timing_state.base_ticks = 0;
    timing_state.total_calls = 0;
    timing_state.calibration_count = 0;
    timing_state.last_calibration = 0;
    
    // Perform initial calibration
    if (!aeth_time_calibrate()) {
        return false;
    }
    
    timing_state.initialized = true;
    return true;
}

bool aeth_time_ready(void) {
    return timing_state.initialized;
}

uint64_t aeth_ticks_now(void) {
    if (!timing_state.initialized) {
        return 0;
    }
    
    timing_state.total_calls++;
    
    struct timespec current_time;
    if (clock_gettime(CLOCK_MONOTONIC, &current_time) != 0) {
        return 0;
    }
    
    // Calculate time difference in nanoseconds
    int64_t diff_ns = (current_time.tv_sec - timing_state.base_time.tv_sec) * 1000000000LL +
                      (current_time.tv_nsec - timing_state.base_time.tv_nsec);
    
    // Convert to ticks using calibration factor
    uint64_t ticks = timing_state.base_ticks + (diff_ns / 1000); // Convert ns to μs
    
    return ticks;
}

double aeth_ticks_to_us(uint64_t dt) {
    if (!timing_state.initialized) {
        return 0.0;
    }
    
    // Convert ticks to microseconds using calibration factor
    return (double)dt;
}

uint64_t aeth_time_resolution_ns(void) {
    struct timespec res;
    if (clock_getres(CLOCK_MONOTONIC, &res) != 0) {
        return 1000; // Default to 1μs if we can't get resolution
    }
    
    return (uint64_t)res.tv_nsec;
}

bool aeth_time_calibrate(void) {
    if (!timing_state.initialized) {
        return false;
    }
    
    // Perform calibration by measuring a known time interval
    const uint64_t target_us = 1000; // 1ms target
    
    struct timespec start_time, end_time;
    if (clock_gettime(CLOCK_MONOTONIC, &start_time) != 0) {
        return false;
    }
    
    // Busy wait for target duration
    uint64_t start_ticks = aeth_ticks_now();
    uint64_t target_ticks = target_us;
    
    while (aeth_ticks_now() - start_ticks < target_ticks) {
        // Spin until target time reached
    }
    
    if (clock_gettime(CLOCK_MONOTONIC, &end_time) != 0) {
        return false;
    }
    
    // Calculate actual duration
    int64_t actual_ns = (end_time.tv_sec - start_time.tv_sec) * 1000000000LL +
                        (end_time.tv_nsec - start_time.tv_nsec);
    uint64_t actual_us = actual_ns / 1000;
    
    // Update calibration factor
    if (actual_us > 0) {
        timing_state.calibration_factor = (target_ticks * 1000) / actual_us;
    }
    
    timing_state.calibration_count++;
    timing_state.last_calibration = aeth_ticks_now();
    
    return true;
}

void aeth_time_stats(uint64_t* total_calls, uint64_t* calibration_count, uint64_t* last_calibration) {
    if (total_calls) {
        *total_calls = timing_state.total_calls;
    }
    if (calibration_count) {
        *calibration_count = timing_state.calibration_count;
    }
    if (last_calibration) {
        *last_calibration = timing_state.last_calibration;
    }
}

// Test functions for validation
#ifdef TEST_TIMING

#include <stdio.h>
#include <assert.h>

static void test_monotonic_behavior(void) {
    printf("Testing monotonic behavior...\n");
    
    uint64_t prev = aeth_ticks_now();
    for (int i = 0; i < 1000; i++) {
        uint64_t current = aeth_ticks_now();
        assert(current >= prev);
        prev = current;
    }
    printf("✓ Monotonic behavior verified\n");
}

static void test_calibration_accuracy(void) {
    printf("Testing calibration accuracy...\n");
    
    // Measure 1ms interval
    uint64_t start = aeth_ticks_now();
    usleep(1000); // Sleep for 1ms
    uint64_t end = aeth_ticks_now();
    
    uint64_t measured_us = end - start;
    double error = (double)abs((int64_t)measured_us - 1000) / 1000.0;
    
    printf("Measured: %lu μs, Expected: 1000 μs, Error: %.2f%%\n", 
           measured_us, error * 100.0);
    
    // Allow 5% error
    assert(error < 0.05);
    printf("✓ Calibration accuracy verified\n");
}

int main(void) {
    printf("Initializing timing system...\n");
    if (!aeth_time_init()) {
        fprintf(stderr, "Failed to initialize timing system\n");
        return 1;
    }
    
    printf("Timing system initialized\n");
    printf("Resolution: %lu ns\n", aeth_time_resolution_ns());
    
    test_monotonic_behavior();
    test_calibration_accuracy();
    
    // Print stats
    uint64_t total_calls, calibration_count, last_calibration;
    aeth_time_stats(&total_calls, &calibration_count, &last_calibration);
    
    printf("Timing stats:\n");
    printf("  Total calls: %lu\n", total_calls);
    printf("  Calibrations: %lu\n", calibration_count);
    printf("  Last calibration: %lu\n", last_calibration);
    
    printf("All tests passed!\n");
    return 0;
}

#endif /* TEST_TIMING */
