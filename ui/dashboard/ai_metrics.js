window.POLYMERA_AI_METRICS = {
  "ai_browser_summaries_total": 0,
  "ai_budget_exhausted_total": 0,
  "ai_capability_denials_total": 0,
  "ai_heg_ddr_pressure_score": 0,
  "ai_heg_decode_to_igpu_total": 0,
  "ai_heg_plans_total": 2,
  "ai_heg_prefill_to_npu_total": 0,
  "ai_hitl_modifies_total": 0,
  "ai_hitl_rejects_total": 0,
  "ai_llm_calls_total": 0,
  "ai_log_summaries_total": 0,
  "ai_ltm_events_total": 0,
  "ai_plans_generated_total": 0,
  "ai_runtime_backend_errors_total": 0,
  "ai_runtime_backend_executions_total": 2,
  "ai_runtime_backend_last_latency_ms": 4,
  "ai_runtime_local_tokens_total": 28,
  "last_browser_summary": null,
  "last_capability_denial": null,
  "last_goal_plan": null,
  "last_heg_plan": {
    "audit_event": "runtime.heg.plan_generated graph_id=heg-903d00d8e4301349 hash=95a5e214e1f16cb65f4938c8d988a5b7a31d2b69e883c4d946a1bc53a9094952 ddr_pressure=0\nruntime.backend.executed backend=DeterministicLocal tokens=14 latency_ms=0 execution_hash=093efc82b754db16a9bfc15832885bf408e9dfa068655ce3d741e20482b13586",
    "backend_kind": "deterministic_local",
    "backend_metadata": {
      "backend_kind": "deterministic_local",
      "execution_mode": "deterministic_local",
      "heg_deterministic_hash": "95a5e214e1f16cb65f4938c8d988a5b7a31d2b69e883c4d946a1bc53a9094952",
      "heg_plan_id": "heg-903d00d8e4301349",
      "model_loaded": "false",
      "placement_summary": "embedding:cpu,prefill:cpu,decode:cpu",
      "processing_time_ms": "4",
      "prompt_hash": "94c07a34b6bf4395d85690884c4bb1f7455e5b6c849d98efbd5d36ef225939e1",
      "remote_execution": "false",
      "tokens_generated": "14"
    },
    "generated_text": "Hello! This is a deterministic local assistant response. How can I help you today?",
    "heg_plan_id": "heg-903d00d8e4301349",
    "placement_summary": [
      {
        "assigned": "cpu",
        "node_id": "embedding",
        "reason": "caller preferred cpu"
      },
      {
        "assigned": "cpu",
        "node_id": "prefill",
        "reason": "caller preferred cpu"
      },
      {
        "assigned": "cpu",
        "node_id": "decode",
        "reason": "caller preferred cpu"
      }
    ],
    "processing_time_ms": 4,
    "tokens_generated": 14
  },
  "last_ltm_event": null,
  "last_summary": null,
  "updated_at_unix": 1779324923
};
