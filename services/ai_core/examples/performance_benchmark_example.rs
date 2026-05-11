/**
 * @file performance_benchmark_example.rs
 * @brief Example demonstrating AI Core Service performance benchmarking
 */

use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Mock types for the example (these would be imported from the actual benchmark module)
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

/// Mock AI Core Service performance client for demonstration
pub struct MockAiCorePerfClient {
    config: PerfBenchConfig,
}

impl MockAiCorePerfClient {
    /// Create a new performance benchmark client
    pub fn new(config: PerfBenchConfig) -> Self {
        Self { config }
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
        let load_time = match self.config.model_config.model_name.as_str() {
            "gpt-3.5-turbo" => 100,
            "gpt-4" => 200,
            "claude-3" => 150,
            _ => 120,
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(load_time)).await;
    }

    /// Simulate prompt preprocessing
    async fn simulate_preprocessing(&self, prompt: &TestPrompt) {
        let base_time = prompt.text.len() as u64 / 100;
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
            PromptComplexity::Simple => 50,
            PromptComplexity::Medium => 100,
            PromptComplexity::Complex => 200,
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(first_token_delay)).await;
        let first_token_time = inference_start.elapsed();
        
        // Simulate token generation
        let total_tokens = prompt.expected_tokens;
        let tokens_per_sec = match prompt.complexity {
            PromptComplexity::Simple => 50.0,
            PromptComplexity::Medium => 30.0,
            PromptComplexity::Complex => 15.0,
        };
        
        let generation_time = (total_tokens as f64 / tokens_per_sec * 1000.0) as u64;
        tokio::time::sleep(tokio::time::Duration::from_millis(generation_time)).await;
        
        let total_inference_time = inference_start.elapsed();
        (first_token_time, total_tokens, total_inference_time)
    }

    /// Simulate post-processing
    async fn simulate_postprocessing(&self) {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    /// Simulate serialization
    async fn simulate_serialization(&self) {
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }

    /// Simulate memory usage
    fn simulate_memory_usage(&self, prompt: &TestPrompt) -> f64 {
        let base_memory = match self.config.model_config.model_name.as_str() {
            "gpt-3.5-turbo" => 2000.0,
            "gpt-4" => 4000.0,
            "claude-3" => 3000.0,
            _ => 2500.0,
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
        match prompt.complexity {
            PromptComplexity::Simple => 25.0,
            PromptComplexity::Medium => 45.0,
            PromptComplexity::Complex => 70.0,
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
        let overall_compliant = compliance_percentage >= 95.0;
        
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

/// Create sample performance benchmark configuration
fn create_sample_config() -> PerfBenchConfig {
    PerfBenchConfig {
        iterations: 10,
        warmup_iterations: 2,
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
            max_first_token_latency_ms: 220,
            min_tokens_per_sec: 18.0,
            max_memory_usage_mb: 3300,
            max_cpu_usage_percent: 88.0,
            max_end_to_end_latency_ms: 5500,
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI Core Service Performance Benchmark Example");
    println!("===============================================");
    
    // Create sample configuration
    let config = create_sample_config();
    println!("✅ Created performance benchmark configuration");
    println!("   Iterations: {}", config.iterations);
    println!("   Test Prompts: {}", config.test_prompts.len());
    println!("   Model: {}", config.model_config.model_name);
    
    // Create performance client
    let client = MockAiCorePerfClient::new(config.clone());
    println!("✅ Created performance benchmark client");
    
    // Run performance benchmarks
    println!("\n🧪 Running Performance Benchmarks:");
    println!("==================================");
    
    let report = client.run_all_benchmarks().await?;
    
    // Print summary
    println!("\n📊 Performance Benchmark Summary:");
    println!("================================");
    println!("Total Benchmarks: {}", report.stats.total_benchmarks);
    println!("Passed Budget: {}", report.stats.passed_budget);
    println!("Failed Budget: {}", report.stats.failed_budget);
    println!("Compliance: {:.1}%", report.budget_compliance.compliance_percentage);
    println!("Overall Compliant: {}", report.budget_compliance.overall_compliant);
    
    // Show detailed results
    println!("\n📋 Detailed Results:");
    println!("===================");
    for result in &report.results {
        let status = if result.passed_budget { "✅" } else { "❌" };
        println!("  {} {}: {}ms first token, {:.1} tokens/sec, {:.1}MB memory, {:.1}% CPU", 
                status, 
                result.prompt_id, 
                result.first_token_latency_ms,
                result.tokens_per_sec,
                result.memory_usage_mb,
                result.cpu_usage_percent);
    }
    
    // Show performance metrics
    println!("\n⏱️  Performance Metrics:");
    println!("=======================");
    println!("Avg First Token Latency: {:.1}ms", report.stats.avg_first_token_latency_ms);
    println!("P95 First Token Latency: {:.1}ms", report.stats.p95_first_token_latency_ms);
    println!("P99 First Token Latency: {:.1}ms", report.stats.p99_first_token_latency_ms);
    println!("Avg Tokens/sec: {:.1}", report.stats.avg_tokens_per_sec);
    println!("P95 Tokens/sec: {:.1}", report.stats.p95_tokens_per_sec);
    println!("P99 Tokens/sec: {:.1}", report.stats.p99_tokens_per_sec);
    println!("Avg Memory Usage: {:.1}MB", report.stats.avg_memory_usage_mb);
    println!("Avg CPU Usage: {:.1}%", report.stats.avg_cpu_usage_percent);
    
    // Show performance margins
    println!("\n📈 Performance Margins:");
    println!("======================");
    println!("First Token Latency Margin: {:.1}ms", report.budget_compliance.performance_margin.first_token_latency_margin_ms);
    println!("Tokens/sec Margin: {:.1}", report.budget_compliance.performance_margin.tokens_per_sec_margin);
    println!("Memory Usage Margin: {:.1}MB", report.budget_compliance.performance_margin.memory_usage_margin_mb);
    println!("CPU Usage Margin: {:.1}%", report.budget_compliance.performance_margin.cpu_usage_margin_percent);
    
    // Show timing breakdown for first result
    if let Some(first_result) = report.results.first() {
        println!("\n🔍 Timing Breakdown ({}):", first_result.prompt_id);
        println!("=========================");
        println!("Model Load: {:.1}ms", first_result.timing_breakdown.model_load_ms);
        println!("Preprocessing: {:.1}ms", first_result.timing_breakdown.preprocessing_ms);
        println!("Inference: {:.1}ms", first_result.timing_breakdown.inference_ms);
        println!("Post-processing: {:.1}ms", first_result.timing_breakdown.postprocessing_ms);
        println!("Serialization: {:.1}ms", first_result.timing_breakdown.serialization_ms);
    }
    
    // Show budget compliance details
    if !report.budget_compliance.failed_constraints.is_empty() {
        println!("\n❌ Failed Constraints:");
        for constraint in &report.budget_compliance.failed_constraints {
            println!("  - {}", constraint);
        }
    }
    
    // Show environment information
    println!("\n🌍 Environment Information:");
    println!("==========================");
    println!("OS: {}", report.environment.os);
    println!("CPU: {}", report.environment.cpu_info);
    println!("Memory: {}", report.environment.memory_info);
    println!("Rust Version: {}", report.environment.rust_version);
    println!("Deterministic: {}", report.environment.deterministic);
    println!("Seed: {}", report.environment.seed);
    
    // Demonstrate performance budget compliance
    println!("\n💰 Performance Budget Compliance:");
    println!("================================");
    println!("Max First Token Latency: {}ms (budget: {}ms)", 
            report.stats.avg_first_token_latency_ms, 
            config.performance_budget.max_first_token_latency_ms);
    println!("Min Tokens/sec: {:.1} (budget: {:.1})", 
            report.stats.avg_tokens_per_sec, 
            config.performance_budget.min_tokens_per_sec);
    println!("Max Memory Usage: {:.1}MB (budget: {}MB)", 
            report.stats.avg_memory_usage_mb, 
            config.performance_budget.max_memory_usage_mb);
    println!("Max CPU Usage: {:.1}% (budget: {:.1}%)", 
            report.stats.avg_cpu_usage_percent, 
            config.performance_budget.max_cpu_usage_percent);
    
    // Save example report
    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write("artifacts/bench/example_performance_report.json", report_json)?;
    println!("\n💾 Example report saved to: artifacts/bench/example_performance_report.json");
    
    println!("\n🎉 Performance benchmark example completed successfully!");
    
    Ok(())
}
