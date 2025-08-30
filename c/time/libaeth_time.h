#ifndef LIB_AETH_TIME_H
#define LIB_AETH_TIME_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Get current monotonic ticks from virtual clock
 * 
 * This function provides a stable, monotonic time source that is
 * consistent with the kernel's virtual clock. The ticks are
 * guaranteed to be monotonically increasing and are not affected
 * by system clock adjustments.
 * 
 * @return Current monotonic ticks (implementation-defined units)
 */
uint64_t aeth_ticks_now(void);

/**
 * Convert tick difference to microseconds
 * 
 * This function converts a difference between two tick values
 * to microseconds. The conversion factor is calibrated once
 * per run to ensure accuracy.
 * 
 * @param dt Tick difference (end - start)
 * @return Time difference in microseconds
 */
double aeth_ticks_to_us(uint64_t dt);

/**
 * Initialize timing system
 * 
 * This function must be called once before using timing functions.
 * It performs calibration and sets up the timing infrastructure.
 * 
 * @return true if initialization successful, false otherwise
 */
bool aeth_time_init(void);

/**
 * Get timing system status
 * 
 * @return true if timing system is ready, false otherwise
 */
bool aeth_time_ready(void);

/**
 * Get timing resolution in nanoseconds
 * 
 * @return Timing resolution in nanoseconds
 */
uint64_t aeth_time_resolution_ns(void);

/**
 * Calibrate timing conversion
 * 
 * This function calibrates the conversion from ticks to microseconds.
 * It should be called periodically to maintain accuracy.
 * 
 * @return true if calibration successful, false otherwise
 */
bool aeth_time_calibrate(void);

/**
 * Get timing statistics
 * 
 * @param total_calls Output: total number of timing calls
 * @param calibration_count Output: number of calibrations performed
 * @param last_calibration Output: timestamp of last calibration
 */
void aeth_time_stats(uint64_t* total_calls, uint64_t* calibration_count, uint64_t* last_calibration);

#ifdef __cplusplus
}
#endif

#endif /* LIB_AETH_TIME_H */
