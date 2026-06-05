window.POLYMERA_AI_METRICS = {
    "ai_browser_summaries_total":  4,
    "ai_budget_exhausted_total":  3,
    "ai_capability_denials_total":  0,
    "ai_heg_ddr_pressure_score":  0,
    "ai_heg_decode_to_igpu_total":  0,
    "ai_heg_plans_total":  5,
    "ai_heg_prefill_to_npu_total":  0,
    "ai_hitl_modifies_total":  3,
    "ai_hitl_rejects_total":  3,
    "ai_llm_calls_total":  0,
    "ai_log_summaries_total":  3,
    "ai_ltm_events_total":  0,
    "ai_plans_generated_total":  7,
    "ai_runtime_backend_errors_total":  0,
    "ai_runtime_backend_executions_total":  5,
    "ai_runtime_backend_last_latency_ms":  5,
    "ai_runtime_local_tokens_total":  112,
    "ai_runtime_model_attempts_total":  0,
    "ai_runtime_model_failures_total":  0,
    "ai_runtime_model_last_latency_ms":  0,
    "ai_runtime_model_success_total":  0,
    "ai_runtime_model_tokens_total":  0,
    "last_browser_summary":  {
                                 "input_chars":  176,
                                 "max_chars":  500,
                                 "metric":  "ai_browser_summaries_total",
                                 "summary":  "Polymera OS operator console\nThe system runs deterministic local AI planning, browser/page assistance,\ncapability-gated log summarization, and a local runtime backend boundary."
                             },
    "last_capability_denial":  null,
    "last_goal_plan":  {
                           "goal":  "Summarize boot logs and plan the next safe kernel boot fix",
                           "learned_backup_drive_unavailable":  false,
                           "plan_id":  "cognitive-daf781656f977bd7",
                           "replay_hash":  "62689e92277d0c506e1696a3da529aa8109f07342b94de9e7b467c7a3f410876",
                           "steps":  [
                                         {
                                             "description":  "understand input",
                                             "group":  0,
                                             "name":  "analyze goal",
                                             "order_in_group":  0,
                                             "step_id":  "step-001"
                                         },
                                         {
                                             "description":  "structure tasks",
                                             "group":  0,
                                             "name":  "plan steps",
                                             "order_in_group":  1,
                                             "step_id":  "step-002"
                                         },
                                         {
                                             "description":  "run actions",
                                             "group":  0,
                                             "name":  "execute plan",
                                             "order_in_group":  2,
                                             "step_id":  "step-003"
                                         },
                                         {
                                             "description":  "understand input",
                                             "group":  1,
                                             "name":  "analyze goal",
                                             "order_in_group":  3,
                                             "step_id":  "step-004"
                                         },
                                         {
                                             "description":  "structure tasks",
                                             "group":  1,
                                             "name":  "plan steps",
                                             "order_in_group":  4,
                                             "step_id":  "step-005"
                                         },
                                         {
                                             "description":  "run actions",
                                             "group":  1,
                                             "name":  "execute plan",
                                             "order_in_group":  5,
                                             "step_id":  "step-006"
                                         }
                                     ]
                       },
    "last_heg_plan":  {
                          "audit_event":  "runtime.heg.plan_generated graph_id=heg-93052d5a155a7e82 hash=6f7e8902b15d82e6a7aaff7d0d6a4813ef69872d17e862f45517788cb33159ee ddr_pressure=102\nruntime.backend.executed backend=DeterministicLocal tokens=28 latency_ms=0 execution_hash=9b72db5575ecf913fc1a8069bd1bbb15bf4bf0586c8cbd525691b957f8078714",
                          "backend_kind":  "deterministic_local",
                          "backend_metadata":  {
                                                   "backend_kind":  "deterministic_local",
                                                   "execution_mode":  "deterministic_local",
                                                   "heg_deterministic_hash":  "6f7e8902b15d82e6a7aaff7d0d6a4813ef69872d17e862f45517788cb33159ee",
                                                   "heg_plan_id":  "heg-93052d5a155a7e82",
                                                   "model_loaded":  "false",
                                                   "output_hash":  "054eea07e8e6787f086d144189a447e590b6395af2428041167c3f2054bba7a3",
                                                   "placement_summary":  "embedding:npu,prefill:npu,decode:igpu,log_summary:cpu",
                                                   "processing_time_ms":  "5",
                                                   "prompt_hash":  "0e9bd398146cb52bef97ebde79604b9aea3e727859780f4927826a9aab1382bf",
                                                   "remote_execution":  "false",
                                                   "tokens_generated":  "28"
                                               },
                          "generated_text":  "Deterministic Local Analysis:\n- Prompt bytes: 71\n- Prompt words: 10\n- Key terms: explain, current, polymera, boot, state, deterministic, paragraph\n- Workload: LogSummary\n- Execution boundary: RuntimeBackend::DeterministicLocal",
                          "heg_plan_id":  "heg-93052d5a155a7e82",
                          "input_hash":  "0e9bd398146cb52bef97ebde79604b9aea3e727859780f4927826a9aab1382bf",
                          "model_id":  null,
                          "model_revision":  null,
                          "model_source":  null,
                          "output_hash":  "054eea07e8e6787f086d144189a447e590b6395af2428041167c3f2054bba7a3",
                          "placement_summary":  [
                                                    {
                                                        "assigned":  "npu",
                                                        "node_id":  "embedding",
                                                        "reason":  "compute-heavy prefill/embedding prefers NPU"
                                                    },
                                                    {
                                                        "assigned":  "npu",
                                                        "node_id":  "prefill",
                                                        "reason":  "compute-heavy prefill/embedding prefers NPU"
                                                    },
                                                    {
                                                        "assigned":  "igpu",
                                                        "node_id":  "decode",
                                                        "reason":  "memory-bound decode prefers iGPU"
                                                    },
                                                    {
                                                        "assigned":  "cpu",
                                                        "node_id":  "log_summary",
                                                        "reason":  "local/capability-gated work stays on CPU"
                                                    }
                                                ],
                          "processing_time_ms":  5,
                          "remote_execution":  false,
                          "tokens_generated":  28
                      },
    "last_hitl_modify":  {
                             "decision":  "modify",
                             "feedback":  "summarize only warnings and errors",
                             "plan_id":  "log-summary-1779395611",
                             "step_id":  "step-001"
                         },
    "last_hitl_reject":  {
                             "decision":  "reject",
                             "plan_id":  "log-summary-1779395611",
                             "step_id":  "step-001"
                         },
    "last_ltm_event":  null,
    "last_phase6_budget_proof":  {
                                     "decision":  "budget_denial_and_refund_test_passed",
                                     "source":  "services/ai_core/tests/phase6_demo_tests.rs"
                                 },
    "last_summary":  {
                         "error_count":  1,
                         "info_count":  2,
                         "line_count":  4,
                         "metric":  "ai_log_summaries_total",
                         "mode":  "deterministic-local-fallback",
                         "source_path":  "C:\\polymera-os\\build_out\\phase4-demo\\sample-runtime.log",
                         "summary":  "4 lines; 1 errors; 1 warnings; 2 info entries; top terms: 2026, 21t00, info, align, allocation, boot",
                         "top_terms":  [
                                           "2026",
                                           "21t00",
                                           "info",
                                           "align",
                                           "allocation",
                                           "boot"
                                       ],
                         "warning_count":  1
                     },
    "updated_at_unix":  1779395623
};
