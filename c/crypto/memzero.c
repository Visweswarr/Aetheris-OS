/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2024 Polymera OS Contributors
 * 
 * Secure memory zeroing implementation
 * 
 * This ensures memory is zeroed even if the compiler optimizes away
 * the operation, which is critical for sensitive data like encryption keys.
 */

#include "polycrypto.h"
#include <string.h>

/* Volatile function pointer to prevent compiler optimization */
static volatile void* (*volatile volatile_memset)(void*, int, size_t) = memset;

void aeth_memzero(void* p, size_t n) {
    if (p == NULL || n == 0) {
        return;
    }
    
    /* Use volatile memset to prevent compiler optimization */
    volatile_memset(p, 0, n);
    
    /* Additional barrier to ensure the operation completes */
    __asm__ volatile("" : : "r"(p) : "memory");
}
