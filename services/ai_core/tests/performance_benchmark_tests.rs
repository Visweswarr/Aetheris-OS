use serde::{Deserialize, Serialize};
/**
 * @file performance_benchmark_tests.rs
 * @brief Tests for AI Core Service performance benchmark system
 */
use std::time::Duration;

// Mock types for testing (these would be imported from the actual benchmark module)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfBenchConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub test_prompts: Vec<TestPrompt>,
    pub model_config: ModelPerfConfig,
    pub performance_budget: PerformanceBudget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPrompt {
    pub id: String,
    pub text: String,
    pub expected_tokens: u32,
    pub complexity: PromptComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptComplexity {
    Simple,
    Medium,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerfConfig {
    pub model_name: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub top_p: f64,
    pub stop_sequences: Vec<String>,
    pub enable_streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBudget {
    pub max_first_token_latency_ms: u64,
    pub min_tokens_per_sec: f64,
    pub max_memory_usage_mb: u64,
    pub max_cpu_usage_percent: f64,
    pub max_end_to_end_latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfBenchResult {
    pub benchmark_id: String,
    pub prompt_id: String,
    pub complexity: PromptComplexity,
    pub first_token_latency_ms: f64,
    pub tokens_per_sec: f64,
    pub total_tokens: u32,
    pub end_to_end_latency_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub passed_budget: bool,
    pub timing_breakdown: TimingBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingBreakdown {
    pub model_load_ms: f64,
    pub preprocessing_ms: f64,
    pub inference_ms: f64,
    pub postprocessing_ms: f64,
    pub serialization_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfBenchReport {
    pub config: PerfBenchConfig,
    pub results: Vec<PerfBenchResult>,
    pub stats: PerfStats,
    pub budget_compliance: BudgetCompliance,
    pub environment: EnvironmentInfo,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfStats {
    pub total_benchmarks: usize,
    pub passed_budget: usize,
    pub failed_budget: usize,
    pub avg_first_token_latency_ms: f64,
    pub p95_first_token_latency_ms: f64,
    pub p99_first_token_latency_ms: f64,
    pub avg_tokens_per_sec: f64,
    pub p95_tokens_per_sec: f64,
    pub p99_tokens_per_sec: f64,
    pub avg_memory_usage_mb: f64,
    pub avg_cpu_usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCompliance {
    pub overall_compliant: bool,
    pub compliance_percentage: f64,
    pub failed_constraints: Vec<String>,
    pub performance_margin: PerformanceMargin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMargin {
    pub first_token_latency_margin_ms: f64,
    pub tokens_per_sec_margin: f64,
    pub memory_usage_margin_mb: f64,
    pub cpu_usage_margin_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub os: String,
    pub cpu_info: String,
    pub memory_info: String,
    pub rust_version: String,
    pub deterministic: bool,
    pub seed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config() -> PerfBenchConfig {
        PerfBenchConfig {
            iterations: 10,
            warmup_iterations: 2,
            test_prompts: vec![
                TestPrompt {
                    id: "test_simple".to_string(),
                    text: "Test simple prompt".to_string(),
                    expected_tokens: 50,
                    complexity: PromptComplexity::Simple,
                },
                TestPrompt {
                    id: "test_complex".to_string(),
                    text: "Test complex prompt with more detailed analysis".to_string(),
                    expected_tokens: 200,
                    complexity: PromptComplexity::Complex,
                },
            ],
            model_config: ModelPerfConfig {
                model_name: "test-model".to_string(),
                temperature: 0.0,
                max_tokens: 1000,
                top_p: 1.0,
                stop_sequences: vec!["\n\n".to_string()],
                enable_streaming: true,
            },
            performance_budget: PerformanceBudget {
                max_first_token_latency_ms: 200,
                min_tokens_per_sec: 20.0,
                max_memory_usage_mb: 3000,
                max_cpu_usage_percent: 80.0,
                max_end_to_end_latency_ms: 5000,
            },
        }
    }

    fn create_test_result() -> PerfBenchResult {
        PerfBenchResult {
            benchmark_id: "test_bench".to_string(),
            prompt_id: "test_prompt".to_string(),
            complexity: PromptComplexity::Simple,
            first_token_latency_ms: 100.0,
            tokens_per_sec: 30.0,
            total_tokens: 50,
            end_to_end_latency_ms: 2000.0,
            memory_usage_mb: 2500.0,
            cpu_usage_percent: 60.0,
            passed_budget: true,
            timing_breakdown: TimingBreakdown {
                model_load_ms: 100.0,
                preprocessing_ms: 10.0,
                inference_ms: 1800.0,
                postprocessing_ms: 5.0,
                serialization_ms: 2.0,
            },
        }
    }

    #[test]
    fn test_perf_bench_config_creation() {
        let config = create_test_config();

        assert_eq!(config.iterations, 10);
        assert_eq!(config.warmup_iterations, 2);
        assert_eq!(config.test_prompts.len(), 2);
        assert_eq!(config.model_config.model_name, "test-model");
        assert_eq!(config.performance_budget.max_first_token_latency_ms, 200);
    }

    #[test]
    fn test_perf_bench_config_serialization() {
        let config = create_test_config();

        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PerfBenchConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.iterations, config.iterations);
        assert_eq!(deserialized.test_prompts.len(), config.test_prompts.len());
        assert_eq!(
            deserialized.model_config.model_name,
            config.model_config.model_name
        );
    }

    #[test]
    fn test_perf_bench_result_creation() {
        let result = create_test_result();

        assert_eq!(result.benchmark_id, "test_bench");
        assert_eq!(result.prompt_id, "test_prompt");
        assert_eq!(result.first_token_latency_ms, 100.0);
        assert_eq!(result.tokens_per_sec, 30.0);
        assert_eq!(result.total_tokens, 50);
        assert!(result.passed_budget);
    }

    #[test]
    fn test_perf_bench_result_serialization() {
        let result = create_test_result();

        // Test JSON serialization
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: PerfBenchResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.benchmark_id, result.benchmark_id);
        assert_eq!(deserialized.prompt_id, result.prompt_id);
        assert_eq!(
            deserialized.first_token_latency_ms,
            result.first_token_latency_ms
        );
        assert_eq!(deserialized.tokens_per_sec, result.tokens_per_sec);
        assert_eq!(deserialized.passed_budget, result.passed_budget);
    }

    #[test]
    fn test_perf_bench_report_creation() {
        let config = create_test_config();
        let result = create_test_result();

        let stats = PerfStats {
            total_benchmarks: 1,
            passed_budget: 1,
            failed_budget: 0,
            avg_first_token_latency_ms: 100.0,
            p95_first_token_latency_ms: 100.0,
            p99_first_token_latency_ms: 100.0,
            avg_tokens_per_sec: 30.0,
            p95_tokens_per_sec: 30.0,
            p99_tokens_per_sec: 30.0,
            avg_memory_usage_mb: 2500.0,
            avg_cpu_usage_percent: 60.0,
        };

        let budget_compliance = BudgetCompliance {
            overall_compliant: true,
            compliance_percentage: 100.0,
            failed_constraints: Vec::new(),
            performance_margin: PerformanceMargin {
                first_token_latency_margin_ms: 100.0,
                tokens_per_sec_margin: 10.0,
                memory_usage_margin_mb: 500.0,
                cpu_usage_margin_percent: 20.0,
            },
        };

        let environment = EnvironmentInfo {
            os: "test".to_string(),
            cpu_info: "test-cpu".to_string(),
            memory_info: "test-memory".to_string(),
            rust_version: "1.0.0".to_string(),
            deterministic: true,
            seed: 42,
        };

        let report = PerfBenchReport {
            config,
            results: vec![result],
            stats,
            budget_compliance,
            environment,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(report.stats.total_benchmarks, 1);
        assert_eq!(report.stats.passed_budget, 1);
        assert_eq!(report.stats.failed_budget, 0);
        assert!(report.budget_compliance.overall_compliant);
        assert_eq!(report.budget_compliance.compliance_percentage, 100.0);
    }

    #[test]
    fn test_perf_bench_report_serialization() {
        let config = create_test_config();
        let result = create_test_result();

        let stats = PerfStats {
            total_benchmarks: 1,
            passed_budget: 1,
            failed_budget: 0,
            avg_first_token_latency_ms: 100.0,
            p95_first_token_latency_ms: 100.0,
            p99_first_token_latency_ms: 100.0,
            avg_tokens_per_sec: 30.0,
            p95_tokens_per_sec: 30.0,
            p99_tokens_per_sec: 30.0,
            avg_memory_usage_mb: 2500.0,
            avg_cpu_usage_percent: 60.0,
        };

        let budget_compliance = BudgetCompliance {
            overall_compliant: true,
            compliance_percentage: 100.0,
            failed_constraints: Vec::new(),
            performance_margin: PerformanceMargin {
                first_token_latency_margin_ms: 100.0,
                tokens_per_sec_margin: 10.0,
                memory_usage_margin_mb: 500.0,
                cpu_usage_margin_percent: 20.0,
            },
        };

        let environment = EnvironmentInfo {
            os: "test".to_string(),
            cpu_info: "test-cpu".to_string(),
            memory_info: "test-memory".to_string(),
            rust_version: "1.0.0".to_string(),
            deterministic: true,
            seed: 42,
        };

        let report = PerfBenchReport {
            config,
            results: vec![result],
            stats,
            budget_compliance,
            environment,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        // Test JSON serialization
        let json = serde_json::to_string_pretty(&report).unwrap();
        let deserialized: PerfBenchReport = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.stats.total_benchmarks,
            report.stats.total_benchmarks
        );
        assert_eq!(deserialized.stats.passed_budget, report.stats.passed_budget);
        assert_eq!(
            deserialized.budget_compliance.overall_compliant,
            report.budget_compliance.overall_compliant
        );
    }

    #[test]
    fn test_perf_bench_config_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        let config = create_test_config();

        // Save config
        let json = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&config_path, json).unwrap();

        // Load config
        let content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: PerfBenchConfig = serde_json::from_str(&content).unwrap();

        assert_eq!(loaded_config.iterations, config.iterations);
        assert_eq!(loaded_config.test_prompts.len(), config.test_prompts.len());
        assert_eq!(
            loaded_config.model_config.model_name,
            config.model_config.model_name
        );
    }

    #[test]
    fn test_perf_bench_report_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let report_path = temp_dir.path().join("test_report.json");

        let config = create_test_config();
        let result = create_test_result();

        let stats = PerfStats {
            total_benchmarks: 1,
            passed_budget: 1,
            failed_budget: 0,
            avg_first_token_latency_ms: 100.0,
            p95_first_token_latency_ms: 100.0,
            p99_first_token_latency_ms: 100.0,
            avg_tokens_per_sec: 30.0,
            p95_tokens_per_sec: 30.0,
            p99_tokens_per_sec: 30.0,
            avg_memory_usage_mb: 2500.0,
            avg_cpu_usage_percent: 60.0,
        };

        let budget_compliance = BudgetCompliance {
            overall_compliant: true,
            compliance_percentage: 100.0,
            failed_constraints: Vec::new(),
            performance_margin: PerformanceMargin {
                first_token_latency_margin_ms: 100.0,
                tokens_per_sec_margin: 10.0,
                memory_usage_margin_mb: 500.0,
                cpu_usage_margin_percent: 20.0,
            },
        };

        let environment = EnvironmentInfo {
            os: "test".to_string(),
            cpu_info: "test-cpu".to_string(),
            memory_info: "test-memory".to_string(),
            rust_version: "1.0.0".to_string(),
            deterministic: true,
            seed: 42,
        };

        let report = PerfBenchReport {
            config,
            results: vec![result],
            stats,
            budget_compliance,
            environment,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        // Save report
        let json = serde_json::to_string_pretty(&report).unwrap();
        fs::write(&report_path, json).unwrap();

        // Load report
        let content = fs::read_to_string(&report_path).unwrap();
        let loaded_report: PerfBenchReport = serde_json::from_str(&content).unwrap();

        assert_eq!(loaded_report.stats.total_benchmarks, 1);
        assert_eq!(loaded_report.stats.passed_budget, 1);
        assert_eq!(loaded_report.results.len(), 1);
        assert!(loaded_report.budget_compliance.overall_compliant);
    }

    #[test]
    fn test_prompt_complexity_enum() {
        let simple = PromptComplexity::Simple;
        let medium = PromptComplexity::Medium;
        let complex = PromptComplexity::Complex;

        // Test serialization
        let simple_json = serde_json::to_string(&simple).unwrap();
        let medium_json = serde_json::to_string(&medium).unwrap();
        let complex_json = serde_json::to_string(&complex).unwrap();

        assert_eq!(simple_json, "\"Simple\"");
        assert_eq!(medium_json, "\"Medium\"");
        assert_eq!(complex_json, "\"Complex\"");

        // Test deserialization
        let deserialized_simple: PromptComplexity = serde_json::from_str(&simple_json).unwrap();
        let deserialized_medium: PromptComplexity = serde_json::from_str(&medium_json).unwrap();
        let deserialized_complex: PromptComplexity = serde_json::from_str(&complex_json).unwrap();

        match deserialized_simple {
            PromptComplexity::Simple => {}
            _ => panic!("Expected Simple variant"),
        }

        match deserialized_medium {
            PromptComplexity::Medium => {}
            _ => panic!("Expected Medium variant"),
        }

        match deserialized_complex {
            PromptComplexity::Complex => {}
            _ => panic!("Expected Complex variant"),
        }
    }

    #[test]
    fn test_performance_budget_validation() {
        let budget = PerformanceBudget {
            max_first_token_latency_ms: 200,
            min_tokens_per_sec: 20.0,
            max_memory_usage_mb: 3000,
            max_cpu_usage_percent: 80.0,
            max_end_to_end_latency_ms: 5000,
        };

        assert_eq!(budget.max_first_token_latency_ms, 200);
        assert_eq!(budget.min_tokens_per_sec, 20.0);
        assert_eq!(budget.max_memory_usage_mb, 3000);
        assert_eq!(budget.max_cpu_usage_percent, 80.0);
        assert_eq!(budget.max_end_to_end_latency_ms, 5000);
    }

    #[test]
    fn test_timing_breakdown_validation() {
        let breakdown = TimingBreakdown {
            model_load_ms: 100.0,
            preprocessing_ms: 10.0,
            inference_ms: 1800.0,
            postprocessing_ms: 5.0,
            serialization_ms: 2.0,
        };

        assert_eq!(breakdown.model_load_ms, 100.0);
        assert_eq!(breakdown.preprocessing_ms, 10.0);
        assert_eq!(breakdown.inference_ms, 1800.0);
        assert_eq!(breakdown.postprocessing_ms, 5.0);
        assert_eq!(breakdown.serialization_ms, 2.0);
    }

    #[test]
    fn test_performance_margin_validation() {
        let margin = PerformanceMargin {
            first_token_latency_margin_ms: 100.0,
            tokens_per_sec_margin: 10.0,
            memory_usage_margin_mb: 500.0,
            cpu_usage_margin_percent: 20.0,
        };

        assert_eq!(margin.first_token_latency_margin_ms, 100.0);
        assert_eq!(margin.tokens_per_sec_margin, 10.0);
        assert_eq!(margin.memory_usage_margin_mb, 500.0);
        assert_eq!(margin.cpu_usage_margin_percent, 20.0);
    }

    #[test]
    fn test_environment_info_validation() {
        let env = EnvironmentInfo {
            os: "Linux".to_string(),
            cpu_info: "Intel Core i7".to_string(),
            memory_info: "16GB DDR4".to_string(),
            rust_version: "1.75.0".to_string(),
            deterministic: true,
            seed: 42,
        };

        assert_eq!(env.os, "Linux");
        assert_eq!(env.cpu_info, "Intel Core i7");
        assert_eq!(env.memory_info, "16GB DDR4");
        assert_eq!(env.rust_version, "1.75.0");
        assert!(env.deterministic);
        assert_eq!(env.seed, 42);
    }

    #[test]
    fn test_budget_compliance_validation() {
        let compliance = BudgetCompliance {
            overall_compliant: true,
            compliance_percentage: 95.5,
            failed_constraints: vec!["Memory usage exceeds budget".to_string()],
            performance_margin: PerformanceMargin {
                first_token_latency_margin_ms: 50.0,
                tokens_per_sec_margin: 5.0,
                memory_usage_margin_mb: -100.0, // Negative margin indicates over budget
                cpu_usage_margin_percent: 10.0,
            },
        };

        assert!(compliance.overall_compliant);
        assert_eq!(compliance.compliance_percentage, 95.5);
        assert_eq!(compliance.failed_constraints.len(), 1);
        assert_eq!(
            compliance.failed_constraints[0],
            "Memory usage exceeds budget"
        );
        assert_eq!(compliance.performance_margin.memory_usage_margin_mb, -100.0);
    }

    #[test]
    fn test_perf_stats_validation() {
        let stats = PerfStats {
            total_benchmarks: 10,
            passed_budget: 9,
            failed_budget: 1,
            avg_first_token_latency_ms: 150.0,
            p95_first_token_latency_ms: 200.0,
            p99_first_token_latency_ms: 250.0,
            avg_tokens_per_sec: 25.0,
            p95_tokens_per_sec: 30.0,
            p99_tokens_per_sec: 35.0,
            avg_memory_usage_mb: 2800.0,
            avg_cpu_usage_percent: 65.0,
        };

        assert_eq!(stats.total_benchmarks, 10);
        assert_eq!(stats.passed_budget, 9);
        assert_eq!(stats.failed_budget, 1);
        assert_eq!(stats.avg_first_token_latency_ms, 150.0);
        assert_eq!(stats.p95_first_token_latency_ms, 200.0);
        assert_eq!(stats.p99_first_token_latency_ms, 250.0);
        assert_eq!(stats.avg_tokens_per_sec, 25.0);
        assert_eq!(stats.p95_tokens_per_sec, 30.0);
        assert_eq!(stats.p99_tokens_per_sec, 35.0);
        assert_eq!(stats.avg_memory_usage_mb, 2800.0);
        assert_eq!(stats.avg_cpu_usage_percent, 65.0);
    }

    #[test]
    fn test_model_perf_config_validation() {
        let model_config = ModelPerfConfig {
            model_name: "gpt-3.5-turbo".to_string(),
            temperature: 0.0,
            max_tokens: 1000,
            top_p: 1.0,
            stop_sequences: vec!["\n\n".to_string(), "Human:".to_string()],
            enable_streaming: true,
        };

        assert_eq!(model_config.model_name, "gpt-3.5-turbo");
        assert_eq!(model_config.temperature, 0.0);
        assert_eq!(model_config.max_tokens, 1000);
        assert_eq!(model_config.top_p, 1.0);
        assert_eq!(model_config.stop_sequences.len(), 2);
        assert!(model_config.enable_streaming);
    }

    #[test]
    fn test_test_prompt_validation() {
        let prompt = TestPrompt {
            id: "test_prompt".to_string(),
            text: "What is the capital of France?".to_string(),
            expected_tokens: 25,
            complexity: PromptComplexity::Simple,
        };

        assert_eq!(prompt.id, "test_prompt");
        assert_eq!(prompt.text, "What is the capital of France?");
        assert_eq!(prompt.expected_tokens, 25);

        match prompt.complexity {
            PromptComplexity::Simple => {}
            _ => panic!("Expected Simple complexity"),
        }
    }
}
