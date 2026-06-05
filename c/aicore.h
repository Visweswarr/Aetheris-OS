#ifndef AICORE_H
#define AICORE_H

#ifdef __cplusplus
extern "C" {
#endif

// Initialize the Cognitive Core with default configuration.
// Returns 0 on success, non-zero on failure.
int aicore_init_default(void);

// Submit a goal to the Cognitive Core planner.
// goal_json: UTF-8 null-terminated string of a JSON object or string goal.
// out_buf: output buffer to receive a JSON-encoded PlanResult.
// out_len: length of out_buf in bytes.
// Returns 0 on success; non-zero error code on failure.
int aicore_submit_goal_json(const char* goal_json, unsigned char* out_buf, unsigned long long out_len);

// Create a snapshot and write the snapshot ID (ASCII hex) into out_id_buf.
int aicore_snapshot(unsigned char* out_id_buf, unsigned long long out_len);

// Replay a snapshot by ID; returns 0 on success.
int aicore_replay(const char* snapshot_id);

#ifdef __cplusplus
}
#endif

#endif // AICORE_H
