use alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec;
use core::fmt::Write;

use super::{FlakyTestResult, TestRunResult};

/// GitHub issue creation request
#[derive(Debug, Clone)]
pub struct IssueRequest {
    /// Issue title
    pub title: String,
    /// Issue body content
    pub body: String,
    /// Issue labels
    pub labels: Vec<String>,
    /// Issue assignees
    pub assignees: Vec<String>,
    /// Issue milestone
    pub milestone: Option<String>,
    /// Attachments (logs, minidumps)
    pub attachments: Vec<Attachment>,
}

/// File attachment for issue
#[derive(Debug, Clone)]
pub struct Attachment {
    /// File name
    pub filename: String,
    /// File content
    pub content: Vec<u8>,
    /// Content type
    pub content_type: String,
}

/// Issue opener configuration
#[derive(Debug, Clone)]
pub struct IssueOpenerConfig {
    /// GitHub repository owner
    pub repo_owner: String,
    /// GitHub repository name
    pub repo_name: String,
    /// GitHub API token
    pub api_token: String,
    /// Issue template path
    pub template_path: String,
    /// Default labels for flaky test issues
    pub default_labels: Vec<String>,
    /// Default assignees for flaky test issues
    pub default_assignees: Vec<String>,
    /// Whether to auto-open issues
    pub auto_open: bool,
    /// Issue creation delay (to avoid spam)
    pub creation_delay_ms: u64,
    /// Maximum variance percentage threshold
    pub max_variance_percent: f64,
}

impl Default for IssueOpenerConfig {
    fn default() -> Self {
        Self {
            repo_owner: String::from("polymera-os"),
            repo_name: String::from("polymera-os"),
            api_token: String::new(), // Will be set from environment
            template_path: String::from(".github/ISSUE_TEMPLATE/flaky_test.md"),
            default_labels: vec![
                String::from("flaky-test"),
                String::from("bug"),
                String::from("test-reliability"),
            ],
            default_assignees: vec![
                String::from("test-team"),
            ],
            auto_open: true,
            creation_delay_ms: 5000, // 5 second delay
            max_variance_percent: 10.0,
        }
    }
}

/// Issue opener for flaky tests
pub struct IssueOpener {
    config: IssueOpenerConfig,
    last_issue_time: core::sync::atomic::AtomicU64,
}

impl IssueOpener {
    /// Create a new issue opener with default configuration
    pub fn new() -> Self {
        Self {
            config: IssueOpenerConfig::default(),
            last_issue_time: core::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Create a new issue opener with custom configuration
    pub fn with_config(config: IssueOpenerConfig) -> Self {
        Self {
            config,
            last_issue_time: core::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Check if enough time has passed since last issue creation
    fn can_create_issue(&self) -> bool {
        let current_time = crate::determinism::get_global_time_ms();
        let last_time = self.last_issue_time.load(core::sync::atomic::Ordering::Relaxed);
        
        current_time.saturating_sub(last_time) >= self.config.creation_delay_ms
    }

    /// Create an issue for a flaky test
    pub fn create_flaky_test_issue(&self, result: &FlakyTestResult) -> Result<String, String> {
        if !self.config.auto_open {
            return Err("Auto-opening issues is disabled".to_string());
        }

        if !self.can_create_issue() {
            return Err("Rate limit: too soon since last issue creation".to_string());
        }

        // Update last issue time
        let current_time = crate::determinism::get_global_time_ms();
        self.last_issue_time.store(current_time, core::sync::atomic::Ordering::Relaxed);

        // Create issue request
        let issue_request = self.build_issue_request(result)?;
        
        // Create the issue
        let issue_url = self.create_github_issue(&issue_request)?;
        
        Ok(issue_url)
    }

    /// Build issue request from flaky test result
    fn build_issue_request(&self, result: &FlakyTestResult) -> Result<IssueRequest, String> {
        let title = format!("Flaky Test: {} ({}% variance)", 
                           result.test_name, 
                           result.variance_percent as u32);
        
        let body = self.generate_issue_body(result)?;
        
        let mut labels = self.config.default_labels.clone();
        labels.push(format!("variance-{}", (result.variance_percent / 10.0) as u32 * 10));
        
        if result.stats.success_rate_percent < 100.0 {
            labels.push(String::from("intermittent-failure"));
        }
        
        let attachments = self.collect_attachments(result)?;
        
        Ok(IssueRequest {
            title,
            body,
            labels,
            assignees: self.config.default_assignees.clone(),
            milestone: None,
            attachments,
        })
    }

    /// Generate issue body content
    fn generate_issue_body(&self, result: &FlakyTestResult) -> Result<String, String> {
        let mut body = String::new();
        
        // Header
        writeln!(body, "## Flaky Test Detected").map_err(|_| "Failed to write header")?;
        writeln!(body).map_err(|_| "Failed to write newline")?;
        
        // Test information
        writeln!(body, "**Test Name**: `{}`", result.test_name).map_err(|_| "Failed to write test name")?;
        writeln!(body, "**Variance**: {:.2}%", result.variance_percent).map_err(|_| "Failed to write variance")?;
        writeln!(body, "**Test Runs**: {}", result.run_results.len()).map_err(|_| "Failed to write test runs")?;
        writeln!(body, "**Success Rate**: {:.1}%", result.stats.success_rate_percent).map_err(|_| "Failed to write success rate")?;
        writeln!(body, "**Threshold**: {:.1}%", self.config.max_variance_percent).map_err(|_| "Failed to write threshold")?;
        writeln!(body).map_err(|_| "Failed to write newline")?;
        
        // Statistics
        writeln!(body, "### Test Statistics").map_err(|_| "Failed to write stats header")?;
        writeln!(body, "- **Mean Duration**: {:.2} ms", result.stats.mean_duration_ms).map_err(|_| "Failed to write mean duration")?;
        writeln!(body, "- **Standard Deviation**: {:.2} ms", result.stats.std_dev_ms).map_err(|_| "Failed to write std dev")?;
        writeln!(body, "- **Min Duration**: {} ms", result.stats.min_duration_ms).map_err(|_| "Failed to write min duration")?;
        writeln!(body, "- **Max Duration**: {} ms", result.stats.max_duration_ms).map_err(|_| "Failed to write max duration")?;
        writeln!(body).map_err(|_| "Failed to write newline")?;
        
        // Test results
        writeln!(body, "### Test Run Results").map_err(|_| "Failed to write results header")?;
        for run in &result.run_results {
            let status = if run.success { "✅ PASS" } else { "❌ FAIL" };
            writeln!(body, "- **Run {}**: {} ({} ms)", 
                    run.run_number, status, run.duration_ms).map_err(|_| "Failed to write run result")?;
            
            if let Some(ref error) = run.error {
                writeln!(body, "  - Error: {}", error).map_err(|_| "Failed to write error")?;
            }
        }
        writeln!(body).map_err(|_| "Failed to write newline")?;
        
        // Recommendations
        if !result.recommendations.is_empty() {
            writeln!(body, "### Recommendations").map_err(|_| "Failed to write recommendations header")?;
            for rec in &result.recommendations {
                writeln!(body, "- {}", rec).map_err(|_| "Failed to write recommendation")?;
            }
            writeln!(body).map_err(|_| "Failed to write newline")?;
        }
        
        // Environment info
        writeln!(body, "### Environment Information").map_err(|_| "Failed to write env header")?;
        writeln!(body, "- **Kernel Version**: {}", self.get_kernel_version()).map_err(|_| "Failed to write kernel version")?;
        writeln!(body, "- **Build Hash**: {}", self.get_build_hash()).map_err(|_| "Failed to write build hash")?;
        writeln!(body, "- **Detection Time**: {}", current_time_formatted()).map_err(|_| "Failed to write detection time")?;
        writeln!(body).map_err(|_| "Failed to write newline")?;
        
        // Attachments info
        if !result.run_results.iter().any(|r| r.minidump.is_some()) {
            writeln!(body, "> **Note**: No minidump data available for this test failure.").map_err(|_| "Failed to write note")?;
        }
        
        Ok(body)
    }

    /// Collect attachments from test results
    fn collect_attachments(&self, result: &FlakyTestResult) -> Result<Vec<Attachment>, String> {
        let mut attachments = Vec::new();
        
        // Collect logs from all runs
        let mut all_logs = String::new();
        for (i, run) in result.run_results.iter().enumerate() {
            writeln!(all_logs, "=== Run {} ===", i + 1).map_err(|_| "Failed to write run header")?;
            writeln!(all_logs, "Success: {}", run.success).map_err(|_| "Failed to write success")?;
            writeln!(all_logs, "Duration: {} ms", run.duration_ms).map_err(|_| "Failed to write duration")?;
            if let Some(ref error) = run.error {
                writeln!(all_logs, "Error: {}", error).map_err(|_| "Failed to write error")?;
            }
            writeln!(all_logs, "Logs:").map_err(|_| "Failed to write logs header")?;
            writeln!(all_logs, "{}", run.logs).map_err(|_| "Failed to write logs")?;
            writeln!(all_logs).map_err(|_| "Failed to write newline")?;
        }
        
        attachments.push(Attachment {
            filename: format!("{}_logs.txt", result.test_name),
            content: all_logs.into_bytes(),
            content_type: String::from("text/plain"),
        });
        
        // Collect minidumps from failed runs
        for (i, run) in result.run_results.iter().enumerate() {
            if let Some(ref minidump) = run.minidump {
                attachments.push(Attachment {
                    filename: format!("{}_run_{}_minidump.bin", result.test_name, i + 1),
                    content: minidump.clone(),
                    content_type: String::from("application/octet-stream"),
                });
            }
        }
        
        // Add test statistics as JSON
        let stats_json = self.serialize_stats_to_json(result)?;
        attachments.push(Attachment {
            filename: format!("{}_stats.json", result.test_name),
            content: stats_json.into_bytes(),
            content_type: String::from("application/json"),
        });
        
        Ok(attachments)
    }

    /// Serialize test statistics to JSON
    fn serialize_stats_to_json(&self, result: &FlakyTestResult) -> Result<String, String> {
        let mut json = String::new();
        
        writeln!(json, "{{").map_err(|_| "Failed to write json start")?;
        writeln!(json, "  \"test_name\": \"{}\",", result.test_name).map_err(|_| "Failed to write test name")?;
        writeln!(json, "  \"is_flaky\": {},", result.is_flaky).map_err(|_| "Failed to write is flaky")?;
        writeln!(json, "  \"variance_percent\": {:.2},", result.variance_percent).map_err(|_| "Failed to write variance")?;
        writeln!(json, "  \"total_runs\": {},", result.run_results.len()).map_err(|_| "Failed to write total runs")?;
        writeln!(json, "  \"successful_runs\": {}", 
                result.run_results.iter().filter(|r| r.success).count()).map_err(|_| "Failed to write successful runs")?;
        writeln!(json, "  \"statistics\": {{").map_err(|_| "Failed to write stats start")?;
        writeln!(json, "    \"mean_duration_ms\": {:.2},", result.stats.mean_duration_ms).map_err(|_| "Failed to write mean duration")?;
        writeln!(json, "    \"std_dev_ms\": {:.2},", result.stats.std_dev_ms).map_err(|_| "Failed to write std dev")?;
        writeln!(json, "    \"coefficient_of_variation\": {:.4},", result.stats.coefficient_of_variation).map_err(|_| "Failed to write coefficient")?;
        writeln!(json, "    \"success_rate_percent\": {:.1},", result.stats.success_rate_percent).map_err(|_| "Failed to write success rate")?;
        writeln!(json, "    \"min_duration_ms\": {},", result.stats.min_duration_ms).map_err(|_| "Failed to write min duration")?;
        writeln!(json, "    \"max_duration_ms\": {}", result.stats.max_duration_ms).map_err(|_| "Failed to write max duration")?;
        writeln!(json, "  }},").map_err(|_| "Failed to write stats end")?;
        writeln!(json, "  \"run_results\": [").map_err(|_| "Failed to write runs start")?;
        
        for (i, run) in result.run_results.iter().enumerate() {
            writeln!(json, "    {{").map_err(|_| "Failed to write run start")?;
            writeln!(json, "      \"run_number\": {},", run.run_number).map_err(|_| "Failed to write run number")?;
            writeln!(json, "      \"duration_ms\": {},", run.duration_ms).map_err(|_| "Failed to write duration")?;
            writeln!(json, "      \"success\": {},", run.success).map_err(|_| "Failed to write success")?;
            if let Some(ref error) = run.error {
                writeln!(json, "      \"error\": \"{}\",", error).map_err(|_| "Failed to write error")?;
            }
            writeln!(json, "      \"has_minidump\": {}", run.minidump.is_some()).map_err(|_| "Failed to write has minidump")?;
            if i < result.run_results.len() - 1 {
                writeln!(json, "    }},").map_err(|_| "Failed to write run end with comma")?;
            } else {
                writeln!(json, "    }}").map_err(|_| "Failed to write run end")?;
            }
        }
        
        writeln!(json, "  ],").map_err(|_| "Failed to write runs end")?;
        writeln!(json, "  \"recommendations\": [").map_err(|_| "Failed to write recommendations start")?;
        
        for (i, rec) in result.recommendations.iter().enumerate() {
            if i < result.recommendations.len() - 1 {
                writeln!(json, "    \"{}\",", rec).map_err(|_| "Failed to write recommendation with comma")?;
            } else {
                writeln!(json, "    \"{}\"", rec).map_err(|_| "Failed to write recommendation")?;
            }
        }
        
        writeln!(json, "  ],").map_err(|_| "Failed to write recommendations end")?;
        writeln!(json, "  \"detection_time\": \"{}\"", current_time_formatted()).map_err(|_| "Failed to write detection time")?;
        writeln!(json, "}}").map_err(|_| "Failed to write json end")?;
        
        Ok(json)
    }

    /// Get kernel version information
    fn get_kernel_version(&self) -> String {
        // This would get the actual kernel version
        // For now, return a placeholder
        String::from("Polymera OS Phase 1.5")
    }

    /// Get kernel build hash
    fn get_build_hash(&self) -> String {
        // This would get the actual build hash
        // For now, return a placeholder
        String::from("unknown")
    }

    /// Create GitHub issue via API
    fn create_github_issue(&self, request: &IssueRequest) -> Result<String, String> {
        // This would make an actual HTTP request to GitHub API
        // For now, simulate the creation and return a placeholder URL
        
        // In a real implementation, this would:
        // 1. Make HTTP POST to https://api.github.com/repos/{owner}/{repo}/issues
        // 2. Include Authorization header with token
        // 3. Upload attachments if any
        // 4. Return the created issue URL
        
        let issue_number = self.generate_issue_number();
        let issue_url = format!("https://github.com/{}/{}/issues/{}", 
                               self.config.repo_owner, 
                               self.config.repo_name, 
                               issue_number);
        
        // Log the issue creation
        crate::kprintln!("Created flaky test issue: {}", issue_url);
        crate::kprintln!("Title: {}", request.title);
        crate::kprintln!("Labels: {:?}", request.labels);
        crate::kprintln!("Attachments: {} files", request.attachments.len());
        
        Ok(issue_url)
    }

    /// Generate a unique issue number
    fn generate_issue_number(&self) -> u64 {
        use core::sync::atomic::{AtomicU64, Ordering};
        static ISSUE_COUNTER: AtomicU64 = AtomicU64::new(1000);
        ISSUE_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    /// Update configuration
    pub fn update_config(&mut self, config: IssueOpenerConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &IssueOpenerConfig {
        &self.config
    }
}

/// Format current time for display
fn current_time_formatted() -> String {
    let current_time = crate::determinism::get_global_time_ms();
    format!("{} ms since boot", current_time)
}

/// Global issue opener instance
static GLOBAL_ISSUE_OPENER: spin::Mutex<Option<IssueOpener>> = spin::Mutex::new(None);

/// Initialize the global issue opener
pub fn init() {
    let mut opener = GLOBAL_ISSUE_OPENER.lock();
    *opener = Some(IssueOpener::new());
}

/// Get the global issue opener
pub fn get_opener() -> Option<spin::MutexGuard<'static, Option<IssueOpener>>> {
    GLOBAL_ISSUE_OPENER.try_lock()
}

/// Create a flaky test issue
pub fn create_flaky_test_issue(result: &FlakyTestResult) -> Result<String, String> {
    if let Some(opener) = get_opener() {
        if let Some(ref opener) = *opener {
            opener.create_flaky_test_issue(result)
        } else {
            Err("Issue opener not initialized".to_string())
        }
    } else {
        Err("Failed to acquire issue opener lock".to_string())
    }
}
