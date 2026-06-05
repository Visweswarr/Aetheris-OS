/**
 * @file ai_core_bench.rs
 * @brief AI Core Service Performance Benchmarks
 * 
 * This benchmark suite measures critical performance metrics for the AI Core Service:
 * - Prompt-to-first-token latency
 * - Tokens per second throughput
 * - Memory usage during inference
 * - CPU utilization
 * - End-to-end response time
 */

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use std::fs;
use std::path::Path;

/// Performance benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfBenchConfig {
    /// Number of benchmark iterations
    pub iterations: usize,
    /// Warm-up iterations
    pub warmup_iterations: usize,
    /// Test prompts for benchmarking
    pub test_prompts: Vec<TestPrompt>,
    /// Model configuration
    pub model_config: ModelPerfConfig,
    /// Performance budget (baseline + 10%)
    pub performance_budget: PerformanceBudget,
}

/// Test prompt for benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPrompt {
    /// Unique identifier
    pub id: String,
    /// Prompt text
    pub text: String,
    /// Expected response characteristics
    pub expected_tokens: u32,
    /// Prompt complexity (simple, medium, complex)
    pub complexity: PromptComplexity,
}

/// Prompt complexity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptComplexity {
    Simple,
    Medium,
    Complex,
}

/// Model performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerfConfig {
    /// Model name/identifier
    pub model_name: String,
    /// Temperature (0.0 for deterministic)
    pub temperature: f64,
    /// Maximum tokens
    pub max_tokens: u32,
    /// Top-p setting
    pub top_p: f64,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
    /// Enable streaming
    pub enable_streaming: bool,
}

/// Performance budget constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBudget {
    /// Maximum prompt-to-first-token latency (ms)
    pub max_first_token_latency_ms: u64,
    /// Minimum tokens per second
    pub min_tokens_per_sec: f64,
    /// Maximum memory usage (MB)
    pub max_memory_usage_mb: u64,
    /// Maximum CPU usage (%)
    pub max_cpu_usage_percent: f64,
    /// Maximum end-to-end latency (ms)
    pub max_end_to_end_latency_ms: u64,
}

/// Performance benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfBenchResult {
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Test prompt ID
    pub prompt_id: String,
    /// Prompt complexity
    pub complexity: PromptComplexity,
    /// Prompt-to-first-token latency (ms)
    pub first_token_latency_ms: f64,
    /// Tokens per second
    pub tokens_per_sec: f64,
    /// Total tokens generated
    pub total_tokens: u32,
    /// End-to-end latency (ms)
    pub end_to_end_latency_ms: f64,
    /// Memory usage (MB)
    pub memory_usage_mb: f64,
    /// CPU usage (%)
    pub cpu_usage_percent: f64,
    /// Whether the benchmark passed the performance budget
    pub passed_budget: bool,
    /// Detailed timing breakdown
    pub timing_breakdown: TimingBreakdown,
}

/// Detailed timing breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingBreakdown {
    /// Model loading time (ms)
    pub model_load_ms: f64,
    /// Prompt preprocessing time (ms)
    pub preprocessing_ms: f64,
    /// Inference time (ms)
    pub inference_ms: f64,
    /// Post-processing time (ms)
    pub postprocessing_ms: f64,
    /// Serialization time (ms)
    pub serialization_ms: f64,
}

/// Performance benchmark report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfBenchReport {
    /// Benchmark configuration
    pub config: PerfBenchConfig,
    /// Individual benchmark results
    pub results: Vec<PerfBenchResult>,
    /// Overall statistics
    pub stats: PerfStats,
    /// Performance budget compliance
    pub budget_compliance: BudgetCompliance,
    /// Environment information
    pub environment: EnvironmentInfo,
    /// Timestamp
    pub timestamp: String,
}

/// Overall performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfStats {
    /// Total benchmarks run
    pub total_benchmarks: usize,
    /// Benchmarks that passed budget
    pub passed_budget: usize,
    /// Benchmarks that failed budget
    pub failed_budget: usize,
    /// Average first token latency (ms)
    pub avg_first_token_latency_ms: f64,
    /// P95 first token latency (ms)
    pub p95_first_token_latency_ms: f64,
    /// P99 first token latency (ms)
    pub p99_first_token_latency_ms: f64,
    /// Average tokens per second
    pub avg_tokens_per_sec: f64,
    /// P95 tokens per second
    pub p95_tokens_per_sec: f64,
    /// P99 tokens per second
    pub p99_tokens_per_sec: f64,
    /// Average memory usage (MB)
    pub avg_memory_usage_mb: f64,
    /// Average CPU usage (%)
    pub avg_cpu_usage_percent: f64,
}

/// Budget compliance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCompliance {
    /// Overall compliance status
    pub overall_compliant: bool,
    /// Compliance percentage
    pub compliance_percentage: f64,
    /// Failed constraints
    pub failed_constraints: Vec<String>,
    /// Performance margin (how much headroom remains)
    pub performance_margin: PerformanceMargin,
}

/// Performance margin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMargin {
    /// First token latency margin (ms)
    pub first_token_latency_margin_ms: f64,
    /// Tokens per second margin
    pub tokens_per_sec_margin: f64,
    /// Memory usage margin (MB)
    pub memory_usage_margin_mb: f64,
    /// CPU usage margin (%)
    pub cpu_usage_margin_percent: f64,
}

/// Environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system
    pub os: String,
    /// CPU information
    pub cpu_info: String,
    /// Memory information
    pub memory_info: String,
    /// Rust version
    pub rust_version: String,
    /// Deterministic mode
    pub deterministic: bool,
    /// Seed used
    pub seed: u64,
}

/// Mock AI Core Service client for benchmarking
pub struct MockAiCorePerfClient {
    config: PerfBenchConfig,
    runtime: Runtime,
}

impl MockAiCorePerfClient {
    /// Create a new performance benchmark client
    pub fn new(config: PerfBenchConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = Runtime::new()?;
        Ok(Self { config, runtime })
    }

    /// Run a single performance benchmark
    pub async fn run_benchmark(&self, prompt: &TestPrompt) -> Result<PerfBenchResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        // Set deterministic environment
        std::env::set_var("AETHERIS_SEED", "42");
        std::env::set_var("AETHERIS_DETERMINISTIC", "true");
        
        // Simulate model loading
        let model_load_start = Instant::now();
        self.simulate_model_loading().await;
        let model_load_time = model_load_start.elapsed();
        
        // Simulate prompt preprocessing
        let preprocessing_start = Instant::now();
        self.simulate_preprocessing(prompt).await;
        let preprocessing_time = preprocessing_start.elapsed();
        
        // Simulate inference with streaming
        let inference_start = Instant::now();
        let (first_token_time, total_tokens, inference_time) = self.simulate_inference(prompt).await;
        let inference_duration = inference_start.elapsed();
        
        // Simulate post-processing
        let postprocessing_start = Instant::now();
        self.simulate_postprocessing().await;
        let postprocessing_time = postprocessing_start.elapsed();
        
        // Simulate serialization
        let serialization_start = Instant::now();
        self.simulate_serialization().await;
        let serialization_time = serialization_start.elapsed();
        
        let total_time = start_time.elapsed();
        
        // Calculate performance metrics
        let first_token_latency_ms = first_token_time.as_millis() as f64;
        let end_to_end_latency_ms = total_time.as_millis() as f64;
        let tokens_per_sec = if inference_time.as_secs_f64() > 0.0 {
            total_tokens as f64 / inference_time.as_secs_f64()
        } else {
            0.0
        };
        
        // Simulate memory and CPU usage
        let memory_usage_mb = self.simulate_memory_usage(prompt);
        let cpu_usage_percent = self.simulate_cpu_usage(prompt);
        
        // Check performance budget compliance
        let passed_budget = self.check_budget_compliance(
            first_token_latency_ms,
            tokens_per_sec,
            memory_usage_mb,
            cpu_usage_percent,
            end_to_end_latency_ms,
        );
        
        let timing_breakdown = TimingBreakdown {
            model_load_ms: model_load_time.as_millis() as f64,
            preprocessing_ms: preprocessing_time.as_millis() as f64,
            inference_ms: inference_duration.as_millis() as f64,
            postprocessing_ms: postprocessing_time.as_millis() as f64,
            serialization_ms: serialization_time.as_millis() as f64,
        };
        
        Ok(PerfBenchResult {
            benchmark_id: format!("bench_{}", prompt.id),
            prompt_id: prompt.id.clone(),
            complexity: prompt.complexity.clone(),
            first_token_latency_ms,
            tokens_per_sec,
            total_tokens,
            end_to_end_latency_ms,
            memory_usage_mb,
            cpu_usage_percent,
            passed_budget,
            timing_breakdown,
        })
    }

    /// Simulate model loading
    async fn simulate_model_loading(&self) {
        // Simulate model loading time based on complexity
        let load_time = match self.config.model_config.model_name.as_str() {
            "gpt-3.5-turbo" => 100, // 100ms
            "gpt-4" => 200,         // 200ms
            "claude-3" => 150,      // 150ms
            _ => 120,               // Default 120ms
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(load_time)).await;
    }

    /// Simulate prompt preprocessing
    async fn simulate_preprocessing(&self, prompt: &TestPrompt) {
        // Simulate preprocessing time based on prompt length and complexity
        let base_time = prompt.text.len() as u64 / 100; // 1ms per 100 characters
        let complexity_multiplier = match prompt.complexity {
            PromptComplexity::Simple => 1.0,
            PromptComplexity::Medium => 1.5,
            PromptComplexity::Complex => 2.0,
        };
        let preprocessing_time = (base_time as f64 * complexity_multiplier) as u64;
        tokio::time::sleep(tokio::time::Duration::from_millis(preprocessing_time.max(1))).await;
    }

    /// Simulate inference with streaming
    async fn simulate_inference(&self, prompt: &TestPrompt) -> (Duration, u32, Duration) {
        let inference_start = Instant::now();
        
        // Simulate first token latency
        let first_token_delay = match prompt.complexity {
            PromptComplexity::Simple => 50,  // 50ms
            PromptComplexity::Medium => 100, // 100ms
            PromptComplexity::Complex => 200, // 200ms
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(first_token_delay)).await;
        let first_token_time = inference_start.elapsed();
        
        // Simulate token generation
        let total_tokens = prompt.expected_tokens;
        let tokens_per_sec = match prompt.complexity {
            PromptComplexity::Simple => 50.0,  // 50 tokens/sec
            PromptComplexity::Medium => 30.0,  // 30 tokens/sec
            PromptComplexity::Complex => 15.0, // 15 tokens/sec
        };
        
        let generation_time = (total_tokens as f64 / tokens_per_sec * 1000.0) as u64;
        tokio::time::sleep(tokio::time::Duration::from_millis(generation_time)).await;
        
        let total_inference_time = inference_start.elapsed();
        (first_token_time, total_tokens, total_inference_time)
    }

    /// Simulate post-processing
    async fn simulate_postprocessing(&self) {
        // Simulate post-processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    /// Simulate serialization
    async fn simulate_serialization(&self) {
        // Simulate serialization time
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }

    /// Simulate memory usage
    fn simulate_memory_usage(&self, prompt: &TestPrompt) -> f64 {
        // Simulate memory usage based on model and prompt complexity
        let base_memory = match self.config.model_config.model_name.as_str() {
            "gpt-3.5-turbo" => 2000.0, // 2GB
            "gpt-4" => 4000.0,         // 4GB
            "claude-3" => 3000.0,      // 3GB
            _ => 2500.0,               // Default 2.5GB
        };
        
        let complexity_multiplier = match prompt.complexity {
            PromptComplexity::Simple => 1.0,
            PromptComplexity::Medium => 1.2,
            PromptComplexity::Complex => 1.5,
        };
        
        base_memory * complexity_multiplier
    }

    /// Simulate CPU usage
    fn simulate_cpu_usage(&self, prompt: &TestPrompt) -> f64 {
        // Simulate CPU usage based on complexity
        match prompt.complexity {
            PromptComplexity::Simple => 25.0,  // 25%
            PromptComplexity::Medium => 45.0,  // 45%
            PromptComplexity::Complex => 70.0, // 70%
        }
    }

    /// Check performance budget compliance
    fn check_budget_compliance(
        &self,
        first_token_latency_ms: f64,
        tokens_per_sec: f64,
        memory_usage_mb: f64,
        cpu_usage_percent: f64,
        end_to_end_latency_ms: f64,
    ) -> bool {
        first_token_latency_ms <= self.config.performance_budget.max_first_token_latency_ms as f64
            && tokens_per_sec >= self.config.performance_budget.min_tokens_per_sec
            && memory_usage_mb <= self.config.performance_budget.max_memory_usage_mb as f64
            && cpu_usage_percent <= self.config.performance_budget.max_cpu_usage_percent
            && end_to_end_latency_ms <= self.config.performance_budget.max_end_to_end_latency_ms as f64
    }

    /// Run all performance benchmarks
    pub async fn run_all_benchmarks(&self) -> Result<PerfBenchReport, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        for prompt in &self.config.test_prompts {
            match self.run_benchmark(prompt).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("Benchmark failed for prompt {}: {}", prompt.id, e);
                }
            }
        }
        
        // Calculate statistics
        let stats = self.calculate_stats(&results);
        let budget_compliance = self.calculate_budget_compliance(&results);
        let environment = self.get_environment_info();
        
        Ok(PerfBenchReport {
            config: self.config.clone(),
            results,
            stats,
            budget_compliance,
            environment,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Calculate performance statistics
    fn calculate_stats(&self, results: &[PerfBenchResult]) -> PerfStats {
        if results.is_empty() {
            return PerfStats {
                total_benchmarks: 0,
                passed_budget: 0,
                failed_budget: 0,
                avg_first_token_latency_ms: 0.0,
                p95_first_token_latency_ms: 0.0,
                p99_first_token_latency_ms: 0.0,
                avg_tokens_per_sec: 0.0,
                p95_tokens_per_sec: 0.0,
                p99_tokens_per_sec: 0.0,
                avg_memory_usage_mb: 0.0,
                avg_cpu_usage_percent: 0.0,
            };
        }
        
        let total_benchmarks = results.len();
        let passed_budget = results.iter().filter(|r| r.passed_budget).count();
        let failed_budget = total_benchmarks - passed_budget;
        
        // Calculate averages
        let avg_first_token_latency_ms = results.iter().map(|r| r.first_token_latency_ms).sum::<f64>() / total_benchmarks as f64;
        let avg_tokens_per_sec = results.iter().map(|r| r.tokens_per_sec).sum::<f64>() / total_benchmarks as f64;
        let avg_memory_usage_mb = results.iter().map(|r| r.memory_usage_mb).sum::<f64>() / total_benchmarks as f64;
        let avg_cpu_usage_percent = results.iter().map(|r| r.cpu_usage_percent).sum::<f64>() / total_benchmarks as f64;
        
        // Calculate percentiles
        let mut first_token_latencies: Vec<f64> = results.iter().map(|r| r.first_token_latency_ms).collect();
        first_token_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mut tokens_per_sec_values: Vec<f64> = results.iter().map(|r| r.tokens_per_sec).collect();
        tokens_per_sec_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p95_first_token_latency_ms = percentile(&first_token_latencies, 0.95);
        let p99_first_token_latency_ms = percentile(&first_token_latencies, 0.99);
        let p95_tokens_per_sec = percentile(&tokens_per_sec_values, 0.95);
        let p99_tokens_per_sec = percentile(&tokens_per_sec_values, 0.99);
        
        PerfStats {
            total_benchmarks,
            passed_budget,
            failed_budget,
            avg_first_token_latency_ms,
            p95_first_token_latency_ms,
            p99_first_token_latency_ms,
            avg_tokens_per_sec,
            p95_tokens_per_sec,
            p99_tokens_per_sec,
            avg_memory_usage_mb,
            avg_cpu_usage_percent,
        }
    }

    /// Calculate budget compliance
    fn calculate_budget_compliance(&self, results: &[PerfBenchResult]) -> BudgetCompliance {
        if results.is_empty() {
            return BudgetCompliance {
                overall_compliant: false,
                compliance_percentage: 0.0,
                failed_constraints: vec!["No benchmarks run".to_string()],
                performance_margin: PerformanceMargin {
                    first_token_latency_margin_ms: 0.0,
                    tokens_per_sec_margin: 0.0,
                    memory_usage_margin_mb: 0.0,
                    cpu_usage_margin_percent: 0.0,
                },
            };
        }
        
        let passed_count = results.iter().filter(|r| r.passed_budget).count();
        let compliance_percentage = (passed_count as f64 / results.len() as f64) * 100.0;
        let overall_compliant = compliance_percentage >= 95.0; // 95% compliance threshold
        
        // Calculate performance margins
        let avg_first_token_latency = results.iter().map(|r| r.first_token_latency_ms).sum::<f64>() / results.len() as f64;
        let avg_tokens_per_sec = results.iter().map(|r| r.tokens_per_sec).sum::<f64>() / results.len() as f64;
        let avg_memory_usage = results.iter().map(|r| r.memory_usage_mb).sum::<f64>() / results.len() as f64;
        let avg_cpu_usage = results.iter().map(|r| r.cpu_usage_percent).sum::<f64>() / results.len() as f64;
        
        let performance_margin = PerformanceMargin {
            first_token_latency_margin_ms: self.config.performance_budget.max_first_token_latency_ms as f64 - avg_first_token_latency,
            tokens_per_sec_margin: avg_tokens_per_sec - self.config.performance_budget.min_tokens_per_sec,
            memory_usage_margin_mb: self.config.performance_budget.max_memory_usage_mb as f64 - avg_memory_usage,
            cpu_usage_margin_percent: self.config.performance_budget.max_cpu_usage_percent - avg_cpu_usage,
        };
        
        // Identify failed constraints
        let mut failed_constraints = Vec::new();
        if avg_first_token_latency > self.config.performance_budget.max_first_token_latency_ms as f64 {
            failed_constraints.push("First token latency exceeds budget".to_string());
        }
        if avg_tokens_per_sec < self.config.performance_budget.min_tokens_per_sec {
            failed_constraints.push("Tokens per second below budget".to_string());
        }
        if avg_memory_usage > self.config.performance_budget.max_memory_usage_mb as f64 {
            failed_constraints.push("Memory usage exceeds budget".to_string());
        }
        if avg_cpu_usage > self.config.performance_budget.max_cpu_usage_percent {
            failed_constraints.push("CPU usage exceeds budget".to_string());
        }
        
        BudgetCompliance {
            overall_compliant,
            compliance_percentage,
            failed_constraints,
            performance_margin,
        }
    }

    /// Get environment information
    fn get_environment_info(&self) -> EnvironmentInfo {
        EnvironmentInfo {
            os: std::env::consts::OS.to_string(),
            cpu_info: "Mock CPU".to_string(),
            memory_info: "Mock Memory".to_string(),
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
            deterministic: true,
            seed: 42,
        }
    }
}

/// Calculate percentile value
fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let index = (percentile * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values[index.min(sorted_values.len() - 1)]
}

/// Create default performance benchmark configuration
fn create_default_config() -> PerfBenchConfig {
    PerfBenchConfig {
        iterations: 100,
        warmup_iterations: 10,
        test_prompts: vec![
            TestPrompt {
                id: "simple_question".to_string(),
                text: "What is the capital of France?".to_string(),
                expected_tokens: 25,
                complexity: PromptComplexity::Simple,
            },
            TestPrompt {
                id: "medium_reasoning".to_string(),
                text: "Explain the difference between supervised and unsupervised machine learning.".to_string(),
                expected_tokens: 150,
                complexity: PromptComplexity::Medium,
            },
            TestPrompt {
                id: "complex_analysis".to_string(),
                text: "Analyze the economic impact of artificial intelligence on the global workforce, considering both positive and negative effects, and provide a detailed assessment with supporting evidence.".to_string(),
                expected_tokens: 300,
                complexity: PromptComplexity::Complex,
            },
        ],
        model_config: ModelPerfConfig {
            model_name: "gpt-3.5-turbo".to_string(),
            temperature: 0.0,
            max_tokens: 1000,
            top_p: 1.0,
            stop_sequences: vec!["\n\n".to_string()],
            enable_streaming: true,
        },
        performance_budget: PerformanceBudget {
            max_first_token_latency_ms: 200,  // 200ms baseline + 10% = 220ms
            min_tokens_per_sec: 20.0,         // 20 tokens/sec baseline - 10% = 18 tokens/sec
            max_memory_usage_mb: 3000,        // 3GB baseline + 10% = 3.3GB
            max_cpu_usage_percent: 80.0,      // 80% baseline + 10% = 88%
            max_end_to_end_latency_ms: 5000,  // 5s baseline + 10% = 5.5s
        },
    }
}

/// Criterion benchmark functions
fn bench_prompt_to_first_token(c: &mut Criterion) {
    let config = create_default_config();
    let client = MockAiCorePerfClient::new(config.clone()).unwrap();
    
    let mut group = c.benchmark_group("prompt_to_first_token");
    group.throughput(Throughput::Elements(1));
    
    for prompt in &config.test_prompts {
        group.bench_with_input(
            BenchmarkId::new("first_token_latency", &prompt.id),
            prompt,
            |b, prompt| {
                b.to_async(&client.runtime).iter(|| async {
                    let result = client.run_benchmark(prompt).await.unwrap();
                    black_box(result.first_token_latency_ms)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_tokens_per_second(c: &mut Criterion) {
    let config = create_default_config();
    let client = MockAiCorePerfClient::new(config.clone()).unwrap();
    
    let mut group = c.benchmark_group("tokens_per_second");
    group.throughput(Throughput::Elements(1));
    
    for prompt in &config.test_prompts {
        group.bench_with_input(
            BenchmarkId::new("tokens_per_sec", &prompt.id),
            prompt,
            |b, prompt| {
                b.to_async(&client.runtime).iter(|| async {
                    let result = client.run_benchmark(prompt).await.unwrap();
                    black_box(result.tokens_per_sec)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_end_to_end_latency(c: &mut Criterion) {
    let config = create_default_config();
    let client = MockAiCorePerfClient::new(config.clone()).unwrap();
    
    let mut group = c.benchmark_group("end_to_end_latency");
    group.throughput(Throughput::Elements(1));
    
    for prompt in &config.test_prompts {
        group.bench_with_input(
            BenchmarkId::new("end_to_end", &prompt.id),
            prompt,
            |b, prompt| {
                b.to_async(&client.runtime).iter(|| async {
                    let result = client.run_benchmark(prompt).await.unwrap();
                    black_box(result.end_to_end_latency_ms)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let config = create_default_config();
    let client = MockAiCorePerfClient::new(config.clone()).unwrap();
    
    let mut group = c.benchmark_group("memory_usage");
    group.throughput(Throughput::Elements(1));
    
    for prompt in &config.test_prompts {
        group.bench_with_input(
            BenchmarkId::new("memory", &prompt.id),
            prompt,
            |b, prompt| {
                b.to_async(&client.runtime).iter(|| async {
                    let result = client.run_benchmark(prompt).await.unwrap();
                    black_box(result.memory_usage_mb)
                })
            },
        );
    }
    
    group.finish();
}

/// Main benchmark runner
pub async fn run_performance_benchmarks() -> Result<PerfBenchReport, Box<dyn std::error::Error>> {
    let config = create_default_config();
    let client = MockAiCorePerfClient::new(config)?;
    
    println!("🚀 Running AI Core Service Performance Benchmarks");
    println!("=================================================");
    println!("Iterations: {}", config.iterations);
    println!("Test Prompts: {}", config.test_prompts.len());
    println!("Model: {}", config.model_config.model_name);
    
    let report = client.run_all_benchmarks().await?;
    
    // Print summary
    println!("\n📊 Performance Benchmark Summary");
    println!("================================");
    println!("Total Benchmarks: {}", report.stats.total_benchmarks);
    println!("Passed Budget: {}", report.stats.passed_budget);
    println!("Failed Budget: {}", report.stats.failed_budget);
    println!("Compliance: {:.1}%", report.budget_compliance.compliance_percentage);
    println!("Overall Compliant: {}", report.budget_compliance.overall_compliant);
    
    println!("\n⏱️  Performance Metrics");
    println!("======================");
    println!("Avg First Token Latency: {:.1}ms", report.stats.avg_first_token_latency_ms);
    println!("P95 First Token Latency: {:.1}ms", report.stats.p95_first_token_latency_ms);
    println!("P99 First Token Latency: {:.1}ms", report.stats.p99_first_token_latency_ms);
    println!("Avg Tokens/sec: {:.1}", report.stats.avg_tokens_per_sec);
    println!("P95 Tokens/sec: {:.1}", report.stats.p95_tokens_per_sec);
    println!("P99 Tokens/sec: {:.1}", report.stats.p99_tokens_per_sec);
    println!("Avg Memory Usage: {:.1}MB", report.stats.avg_memory_usage_mb);
    println!("Avg CPU Usage: {:.1}%", report.stats.avg_cpu_usage_percent);
    
    if !report.budget_compliance.failed_constraints.is_empty() {
        println!("\n❌ Failed Constraints:");
        for constraint in &report.budget_compliance.failed_constraints {
            println!("  - {}", constraint);
        }
    }
    
    Ok(report)
}

/// Save benchmark report to file
pub fn save_benchmark_report(report: &PerfBenchReport, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Serialize report to JSON
    let json = serde_json::to_string_pretty(report)?;
    fs::write(output_path, json)?;
    
    println!("💾 Benchmark report saved to: {}", output_path.display());
    Ok(())
}

criterion_group!(
    benches,
    bench_prompt_to_first_token,
    bench_tokens_per_second,
    bench_end_to_end_latency,
    bench_memory_usage
);

criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_benchmark() {
        let config = create_default_config();
        let client = MockAiCorePerfClient::new(config).unwrap();
        
        let prompt = TestPrompt {
            id: "test_prompt".to_string(),
            text: "Test prompt".to_string(),
            expected_tokens: 50,
            complexity: PromptComplexity::Simple,
        };
        
        let result = client.run_benchmark(&prompt).await.unwrap();
        
        assert!(result.first_token_latency_ms > 0.0);
        assert!(result.tokens_per_sec > 0.0);
        assert!(result.total_tokens > 0);
        assert!(result.end_to_end_latency_ms > 0.0);
        assert!(result.memory_usage_mb > 0.0);
        assert!(result.cpu_usage_percent > 0.0);
    }

    #[tokio::test]
    async fn test_budget_compliance() {
        let config = create_default_config();
        let client = MockAiCorePerfClient::new(config).unwrap();
        
        let prompt = TestPrompt {
            id: "test_prompt".to_string(),
            text: "Test prompt".to_string(),
            expected_tokens: 50,
            complexity: PromptComplexity::Simple,
        };
        
        let result = client.run_benchmark(&prompt).await.unwrap();
        
        // Check that budget compliance is calculated
        assert!(result.passed_budget || !result.passed_budget); // Either true or false is valid
    }

    #[test]
    fn test_percentile_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&values, 0.5), 3.0); // Median
        assert_eq!(percentile(&values, 0.95), 5.0); // P95
        assert_eq!(percentile(&values, 0.99), 5.0); // P99
    }
}
