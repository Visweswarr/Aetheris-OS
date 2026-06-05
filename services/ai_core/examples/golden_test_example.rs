use serde::{Deserialize, Serialize};
/**
 * @file golden_test_example.rs
 * @brief Example demonstrating AI Core Service golden test system
 */
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// Mock types for the example (these would be imported from the actual golden test module)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTestConfig {
    pub seed: u64,
    pub prompts: Vec<TestPrompt>,
    pub model_config: ModelConfig,
    pub expected_outputs: HashMap<String, ExpectedOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPrompt {
    pub id: String,
    pub text: String,
    pub expected_response: PromptExpectation,
    pub expected_tool_calls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptExpectation {
    pub min_length: usize,
    pub max_length: usize,
    pub keywords: Vec<String>,
    pub response_type: ResponseType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseType {
    Text,
    ToolCall,
    Code,
    Structured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_name: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub top_p: f64,
    pub stop_sequences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutput {
    pub response_hash: String,
    pub token_count: u32,
    pub processing_time_ms: u64,
    pub tool_calls: Vec<String>,
    pub expected_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTestResult {
    pub test_id: String,
    pub prompt_id: String,
    pub passed: bool,
    pub actual_response_hash: String,
    pub expected_response_hash: String,
    pub actual_token_count: u32,
    pub expected_token_count: u32,
    pub actual_processing_time_ms: u64,
    pub expected_processing_time_ms: u64,
    pub actual_tool_calls: Vec<String>,
    pub expected_tool_calls: Vec<String>,
    pub error_message: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTestReport {
    pub config: GoldenTestConfig,
    pub results: Vec<GoldenTestResult>,
    pub stats: TestStats,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStats {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f64,
    pub avg_processing_time_ms: f64,
    pub total_processing_time_ms: u64,
}

/// Mock AI Core Service client for demonstration
pub struct MockAiCoreClient {
    config: GoldenTestConfig,
}

impl MockAiCoreClient {
    pub fn new(config: GoldenTestConfig) -> Self {
        Self { config }
    }

    /// Simulate running a golden test
    pub async fn run_test(&self, prompt: &TestPrompt) -> GoldenTestResult {
        // Set deterministic seed
        std::env::set_var("AETHERIS_SEED", self.config.seed.to_string());
        std::env::set_var("AETHERIS_DETERMINISTIC", "true");

        let start_time = SystemTime::now();

        // Simulate AI Core Service processing
        let response = self.simulate_ai_response(prompt).await;

        let processing_time = start_time.elapsed().unwrap().as_millis() as u64;

        // Calculate response hash (deterministic)
        let response_hash = self.calculate_response_hash(&response, prompt);

        // Get expected output
        let expected = self
            .config
            .expected_outputs
            .get(&prompt.id)
            .cloned()
            .unwrap_or_else(|| ExpectedOutput {
                response_hash: "unknown".to_string(),
                token_count: 0,
                processing_time_ms: 0,
                tool_calls: Vec::new(),
                expected_error: Some("No expected output".to_string()),
            });

        // Compare results
        let passed = response_hash == expected.response_hash
            && response.tokens_generated == expected.token_count
            && response.tool_calls == expected.tool_calls;

        GoldenTestResult {
            test_id: format!("test_{}", prompt.id),
            prompt_id: prompt.id.clone(),
            passed,
            actual_response_hash: response_hash,
            expected_response_hash: expected.response_hash,
            actual_token_count: response.tokens_generated,
            expected_token_count: expected.token_count,
            actual_processing_time_ms: processing_time,
            expected_processing_time_ms: expected.processing_time_ms,
            actual_tool_calls: response.tool_calls,
            expected_tool_calls: expected.tool_calls,
            error_message: if passed {
                None
            } else {
                Some("Test failed".to_string())
            },
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Simulate AI response generation
    async fn simulate_ai_response(&self, prompt: &TestPrompt) -> MockResponse {
        // Simulate processing delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Generate deterministic response based on prompt
        let response_text = match prompt.id.as_str() {
            "simple_question" => "The capital of France is Paris.".to_string(),
            "math_problem" => "15 + 27 = 42".to_string(),
            "code_generation" => "def factorial(n):\n    if n <= 1:\n        return 1\n    return \
                                  n * factorial(n-1)"
                .to_string(),
            "file_operation" => {
                "File 'test.txt' created successfully with content 'Hello World'.".to_string()
            }
            "search_query" => {
                "Found information about artificial intelligence in the knowledge base.".to_string()
            }
            "complex_reasoning" => "Supervised learning uses labeled data to train models, while \
                                    unsupervised learning finds patterns in unlabeled data. \
                                    Examples include classification (supervised) and clustering \
                                    (unsupervised)."
                .to_string(),
            "system_command" => "Current system time: 2024-01-15 10:30:45 UTC".to_string(),
            "memory_operation" => "Remembered: Your favorite color is blue.".to_string(),
            "error_handling" => "Error: Division by zero is undefined in mathematics.".to_string(),
            "multimodal_request" => {
                "I can see an image, but I cannot process it in this demonstration.".to_string()
            }
            _ => "Unknown prompt type".to_string(),
        };

        let tool_calls = match prompt.id.as_str() {
            "file_operation" => vec!["create_file".to_string()],
            "search_query" => vec!["search_files".to_string()],
            "system_command" => vec!["get_system_time".to_string()],
            "memory_operation" => vec!["mem.put".to_string()],
            _ => Vec::new(),
        };

        MockResponse {
            tokens_generated: (response_text.len() / 4) as u32, // Rough token estimate
            content: response_text,
            tool_calls,
        }
    }

    /// Calculate deterministic response hash
    fn calculate_response_hash(&self, response: &MockResponse, prompt: &TestPrompt) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        response.content.hash(&mut hasher);
        response.tokens_generated.hash(&mut hasher);
        response.tool_calls.hash(&mut hasher);
        prompt.id.hash(&mut hasher);
        self.config.seed.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}

#[derive(Debug, Clone)]
struct MockResponse {
    content: String,
    tokens_generated: u32,
    tool_calls: Vec<String>,
}

/// Create a sample golden test configuration
fn create_sample_config() -> GoldenTestConfig {
    let mut expected_outputs = HashMap::new();

    // Add expected outputs for each test prompt
    let prompts = vec![
        (
            "simple_question",
            "The capital of France is Paris.",
            25,
            vec![],
        ),
        ("math_problem", "15 + 27 = 42", 15, vec![]),
        (
            "code_generation",
            "def factorial(n):\n    if n <= 1:\n        return 1\n    return n * factorial(n-1)",
            120,
            vec![],
        ),
        (
            "file_operation",
            "File 'test.txt' created successfully with content 'Hello World'.",
            45,
            vec!["create_file"],
        ),
        (
            "search_query",
            "Found information about artificial intelligence in the knowledge base.",
            80,
            vec!["search_files"],
        ),
        (
            "complex_reasoning",
            "Supervised learning uses labeled data to train models, while unsupervised learning \
             finds patterns in unlabeled data. Examples include classification (supervised) and \
             clustering (unsupervised).",
            200,
            vec![],
        ),
        (
            "system_command",
            "Current system time: 2024-01-15 10:30:45 UTC",
            30,
            vec!["get_system_time"],
        ),
        (
            "memory_operation",
            "Remembered: Your favorite color is blue.",
            35,
            vec!["mem.put"],
        ),
        (
            "error_handling",
            "Error: Division by zero is undefined in mathematics.",
            40,
            vec![],
        ),
        (
            "multimodal_request",
            "I can see an image, but I cannot process it in this demonstration.",
            90,
            vec![],
        ),
    ];

    for (id, content, tokens, tool_calls) in prompts {
        // Calculate expected hash (this would be the actual hash from a real run)
        let expected_hash = format!("expected_hash_{}", id);

        expected_outputs.insert(
            id.to_string(),
            ExpectedOutput {
                response_hash: expected_hash,
                token_count: tokens,
                processing_time_ms: 150 + (tokens as u64 * 2), // Simulate processing time
                tool_calls: tool_calls.into_iter().map(String::from).collect(),
                expected_error: None,
            },
        );
    }

    GoldenTestConfig {
        seed: 42,
        prompts: vec![
            TestPrompt {
                id: "simple_question".to_string(),
                text: "What is the capital of France?".to_string(),
                expected_response: PromptExpectation {
                    min_length: 10,
                    max_length: 100,
                    keywords: vec![
                        "Paris".to_string(),
                        "France".to_string(),
                        "capital".to_string(),
                    ],
                    response_type: ResponseType::Text,
                },
                expected_tool_calls: vec![],
            },
            TestPrompt {
                id: "math_problem".to_string(),
                text: "What is 15 + 27?".to_string(),
                expected_response: PromptExpectation {
                    min_length: 5,
                    max_length: 50,
                    keywords: vec!["42".to_string()],
                    response_type: ResponseType::Text,
                },
                expected_tool_calls: vec![],
            },
            TestPrompt {
                id: "code_generation".to_string(),
                text: "Write a simple Python function to calculate the factorial of a number."
                    .to_string(),
                expected_response: PromptExpectation {
                    min_length: 50,
                    max_length: 500,
                    keywords: vec![
                        "def".to_string(),
                        "factorial".to_string(),
                        "python".to_string(),
                        "function".to_string(),
                    ],
                    response_type: ResponseType::Code,
                },
                expected_tool_calls: vec![],
            },
            TestPrompt {
                id: "file_operation".to_string(),
                text: "Create a new file called 'test.txt' with the content 'Hello World'."
                    .to_string(),
                expected_response: PromptExpectation {
                    min_length: 20,
                    max_length: 200,
                    keywords: vec![
                        "file".to_string(),
                        "created".to_string(),
                        "test.txt".to_string(),
                    ],
                    response_type: ResponseType::ToolCall,
                },
                expected_tool_calls: vec!["create_file".to_string()],
            },
            TestPrompt {
                id: "search_query".to_string(),
                text: "Search for information about artificial intelligence.".to_string(),
                expected_response: PromptExpectation {
                    min_length: 30,
                    max_length: 300,
                    keywords: vec![
                        "artificial".to_string(),
                        "intelligence".to_string(),
                        "AI".to_string(),
                    ],
                    response_type: ResponseType::ToolCall,
                },
                expected_tool_calls: vec!["search_files".to_string()],
            },
            TestPrompt {
                id: "complex_reasoning".to_string(),
                text: "Explain the difference between supervised and unsupervised machine \
                       learning, and provide examples of each."
                    .to_string(),
                expected_response: PromptExpectation {
                    min_length: 100,
                    max_length: 800,
                    keywords: vec![
                        "supervised".to_string(),
                        "unsupervised".to_string(),
                        "machine learning".to_string(),
                        "examples".to_string(),
                    ],
                    response_type: ResponseType::Text,
                },
                expected_tool_calls: vec![],
            },
            TestPrompt {
                id: "system_command".to_string(),
                text: "Show me the current system time.".to_string(),
                expected_response: PromptExpectation {
                    min_length: 10,
                    max_length: 100,
                    keywords: vec![
                        "time".to_string(),
                        "current".to_string(),
                        "system".to_string(),
                    ],
                    response_type: ResponseType::ToolCall,
                },
                expected_tool_calls: vec!["get_system_time".to_string()],
            },
            TestPrompt {
                id: "memory_operation".to_string(),
                text: "Remember that my favorite color is blue.".to_string(),
                expected_response: PromptExpectation {
                    min_length: 15,
                    max_length: 100,
                    keywords: vec![
                        "remembered".to_string(),
                        "favorite".to_string(),
                        "color".to_string(),
                        "blue".to_string(),
                    ],
                    response_type: ResponseType::ToolCall,
                },
                expected_tool_calls: vec!["mem.put".to_string()],
            },
            TestPrompt {
                id: "error_handling".to_string(),
                text: "Divide by zero: 5 / 0".to_string(),
                expected_response: PromptExpectation {
                    min_length: 20,
                    max_length: 150,
                    keywords: vec![
                        "error".to_string(),
                        "division".to_string(),
                        "zero".to_string(),
                        "undefined".to_string(),
                    ],
                    response_type: ResponseType::Text,
                },
                expected_tool_calls: vec![],
            },
            TestPrompt {
                id: "multimodal_request".to_string(),
                text: "Describe what you see in this image: [IMAGE_PLACEHOLDER]".to_string(),
                expected_response: PromptExpectation {
                    min_length: 30,
                    max_length: 400,
                    keywords: vec![
                        "image".to_string(),
                        "see".to_string(),
                        "describe".to_string(),
                    ],
                    response_type: ResponseType::Text,
                },
                expected_tool_calls: vec![],
            },
        ],
        model_config: ModelConfig {
            model_name: "gpt-3.5-turbo".to_string(),
            temperature: 0.0,
            max_tokens: 1000,
            top_p: 1.0,
            stop_sequences: vec![
                "\n\n".to_string(),
                "Human:".to_string(),
                "Assistant:".to_string(),
            ],
        },
        expected_outputs,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI Core Service Golden Test Example");
    println!("======================================");

    // Create sample configuration
    let config = create_sample_config();
    println!(
        "✅ Created golden test configuration with {} prompts",
        config.prompts.len()
    );

    // Create mock AI Core client
    let client = MockAiCoreClient::new(config.clone());
    println!("✅ Created mock AI Core client");

    // Run golden tests
    println!("\n🧪 Running Golden Tests:");
    println!("========================");

    let mut results = Vec::new();
    let mut total_processing_time = 0u64;

    for prompt in &config.prompts {
        println!("  Testing: {}", prompt.id);
        let result = client.run_test(prompt).await;
        total_processing_time += result.actual_processing_time_ms;
        results.push(result);
    }

    // Calculate statistics
    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;
    let success_rate = if total_tests > 0 {
        (passed_tests as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };
    let avg_processing_time = if total_tests > 0 {
        total_processing_time as f64 / total_tests as f64
    } else {
        0.0
    };

    let stats = TestStats {
        total_tests,
        passed_tests,
        failed_tests,
        success_rate,
        avg_processing_time_ms: avg_processing_time,
        total_processing_time_ms: total_processing_time,
    };

    // Create test report
    let report = GoldenTestReport {
        config,
        results,
        stats,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    // Print summary
    println!("\n📊 Golden Test Summary:");
    println!("=======================");
    println!("Total Tests: {}", report.stats.total_tests);
    println!("Passed: {}", report.stats.passed_tests);
    println!("Failed: {}", report.stats.failed_tests);
    println!("Success Rate: {:.1}%", report.stats.success_rate);
    println!(
        "Average Processing Time: {:.1}ms",
        report.stats.avg_processing_time_ms
    );

    // Show detailed results
    println!("\n📋 Detailed Results:");
    println!("===================");
    for result in &report.results {
        let status = if result.passed { "✅" } else { "❌" };
        println!(
            "  {} {}: {} tokens, {}ms",
            status, result.prompt_id, result.actual_token_count, result.actual_processing_time_ms
        );

        if !result.passed {
            println!("    Expected hash: {}", result.expected_response_hash);
            println!("    Actual hash:   {}", result.actual_response_hash);
            if let Some(error) = &result.error_message {
                println!("    Error: {}", error);
            }
        }
    }

    // Demonstrate deterministic behavior
    println!("\n🎲 Deterministic Behavior Test:");
    println!("===============================");

    // Run the same test twice with the same seed
    let client1 = MockAiCoreClient::new(create_sample_config());
    let client2 = MockAiCoreClient::new(create_sample_config());

    let prompt = &client1.config.prompts[0]; // Use first prompt
    let result1 = client1.run_test(prompt).await;
    let result2 = client2.run_test(prompt).await;

    println!("  Test: {}", prompt.id);
    println!("  Run 1 hash: {}", result1.actual_response_hash);
    println!("  Run 2 hash: {}", result2.actual_response_hash);

    if result1.actual_response_hash == result2.actual_response_hash {
        println!("  ✅ Deterministic behavior verified - identical hashes");
    } else {
        println!("  ❌ Non-deterministic behavior detected - different hashes");
    }

    // Demonstrate rebaseline scenario
    println!("\n🔄 Rebaseline Scenario:");
    println!("=======================");

    // Simulate a scenario where we need to rebaseline
    let mut rebaseline_config = create_sample_config();

    // Change the expected output for one test
    if let Some(expected) = rebaseline_config
        .expected_outputs
        .get_mut("simple_question")
    {
        expected.response_hash = "new_expected_hash_123".to_string();
        expected.token_count = 30;
    }

    let rebaseline_client = MockAiCoreClient::new(rebaseline_config);
    let rebaseline_result = rebaseline_client
        .run_test(&rebaseline_client.config.prompts[0])
        .await;

    println!("  Test: {}", rebaseline_result.prompt_id);
    println!(
        "  Expected hash: {}",
        rebaseline_result.expected_response_hash
    );
    println!(
        "  Actual hash:   {}",
        rebaseline_result.actual_response_hash
    );
    println!("  Passed: {}", rebaseline_result.passed);

    if !rebaseline_result.passed {
        println!("  💡 This test would fail and require rebaselining");
        println!("  💡 Run: make golden-ai-core-rebaseline");
    }

    // Save example report
    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write("artifacts/ai_core/example_golden_report.json", report_json)?;
    println!("\n💾 Example report saved to: artifacts/ai_core/example_golden_report.json");

    println!("\n🎉 Golden test example completed successfully!");

    Ok(())
}
