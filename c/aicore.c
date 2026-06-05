#include <stdio.h>
#include <string.h>
#include "aicore.h"

// Example usage helper for C callers
int aicore_submit_goal_string(const char* goal, char* out_json, unsigned long long out_len) {
    if (!goal || !out_json || out_len == 0) return 2;
    // Wrap the goal as a JSON string
    char goal_json[1024];
    snprintf(goal_json, sizeof(goal_json), "\"%s\"", goal);
    return aicore_submit_goal_json(goal_json, (unsigned char*)out_json, out_len);
}
