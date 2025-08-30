//! SLO (Service Level Objective) Checker for Polymera OS
//! 
//! This tool validates performance metrics against defined SLOs and fails CI builds
//! when performance targets are not met.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// SLO configuration loaded from YAML
#[derive(Debug, Deserialize)]
struct SloConfig {
    metadata: SloMetadata,
    slo_config: SloSettings,
    slos: HashMap<String, Slo>,
    alerting: Option<AlertingConfig>,
    reporting: Option<ReportingConfig>,
    environments: Option<HashMap<String, EnvironmentConfig>>,
    exemptions: Option<ExemptionsConfig>,
    integrations: Option<IntegrationsConfig>,
    validation: Option<ValidationConfig>,
}

#[derive(Debug, Deserialize)]
struct SloMetadata {
    version: String,
    description: String,
    owner: String,
    updated: String,
}

#[derive(Debug, Deserialize)]
struct SloSettings {
    enforcement_mode: String,
    tolerance_threshold: f64,
    min_samples: u32,
    measurement_window: u32,
}

#[derive(Debug, Deserialize)]
struct Slo {
    name: String,
    description: String,
    metric_name: String,
    metric_type: String,
    targets: SloTargets,
    constraints: Option<HashMap<String, f64>>,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SloTargets {
    p95: Option<f64>,
    p99: Option<f64>,
    p50: Option<f64>,
    max: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct AlertingConfig {
    slack_webhook: Option<String>,
    email_recipients: Option<Vec<String>>,
    severity_levels: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize)]
struct ReportingConfig {
    generate_reports: bool,
    formats: Vec<String>,
    retention_days: u32,
    dashboard: Option<DashboardConfig>,
}

#[derive(Debug, Deserialize)]
struct DashboardConfig {
    grafana_url: Option<String>,
    prometheus_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EnvironmentConfig {
    slo_multiplier: Option<f64>,
    enforcement_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExemptionsConfig {
    temporary: Option<Vec<TemporaryExemption>>,
    permanent: Option<Vec<PermanentExemption>>,
}

#[derive(Debug, Deserialize)]
struct TemporaryExemption {
    slo: String,
    reason: String,
    expires: String,
    approved_by: String,
}

#[derive(Debug, Deserialize)]
struct PermanentExemption {
    slo: String,
    condition: String,
    multiplier: f64,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct IntegrationsConfig {
    prometheus: Option<PrometheusConfig>,
    opentelemetry: Option<OpenTelemetryConfig>,
    custom_backends: Option<Vec<CustomBackend>>,
}

#[derive(Debug, Deserialize)]
struct PrometheusConfig {
    enabled: bool,
    metrics_endpoint: String,
    scrape_interval: String,
}

#[derive(Debug, Deserialize)]
struct OpenTelemetryConfig {
    enabled: bool,
    jaeger_endpoint: String,
    sampling_rate: f64,
}

#[derive(Debug, Deserialize)]
struct CustomBackend {
    name: String,
    r#type: String,
    endpoint: Option<String>,
    connection_string: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ValidationConfig {
    require_minimum_samples: bool,
    validate_timestamps: bool,
    max_age_seconds: u32,
    consistency_checks: Vec<String>,
}

/// Metrics data loaded from JSON
#[derive(Debug, Deserialize)]
struct MetricsData {
    timestamp: DateTime<Utc>,
    environment: String,
    metrics: HashMap<String, MetricValue>,
}

#[derive(Debug, Deserialize)]
struct MetricValue {
    metric_type: String,
    samples: u32,
    values: HashMap<String, f64>,
    metadata: Option<HashMap<String, serde_json::Value>>,
}

/// SLO check result
#[derive(Debug, Serialize)]
struct SloCheckResult {
    slo_name: String,
    status: SloStatus,
    target_value: f64,
    actual_value: f64,
    tolerance_applied: bool,
    exemption_applied: Option<String>,
    violation_severity: Option<String>,
    message: String,
}

#[derive(Debug, Serialize)]
enum SloStatus {
    Pass,
    Fail,
    Warning,
    Exempt,
}

/// Overall check summary
#[derive(Debug, Serialize)]
struct CheckSummary {
    total_slos: u32,
    passed: u32,
    failed: u32,
    warnings: u32,
    exemptions: u32,
    environment: String,
    timestamp: DateTime<Utc>,
    results: Vec<SloCheckResult>,
}

/// SLO Checker implementation
struct SloChecker {
    config: SloConfig,
    environment: String,
    tolerance_mode: bool,
}

impl SloChecker {
    /// Create a new SLO checker
    fn new(config: SloConfig, environment: String, tolerance_mode: bool) -> Self {
        Self {
            config,
            environment,
            tolerance_mode,
        }
    }

    /// Check all SLOs against provided metrics
    fn check_slos(&self, metrics: &MetricsData) -> Result<CheckSummary> {
        info!("Starting SLO checks for environment: {}", self.environment);
        
        let mut results = Vec::new();
        let mut passed = 0;
        let mut failed = 0;
        let mut warnings = 0;
        let mut exemptions = 0;

        for (slo_id, slo) in &self.config.slos {
            debug!("Checking SLO: {} ({})", slo_id, slo.name);
            
            match self.check_individual_slo(slo_id, slo, metrics) {
                Ok(result) => {
                    match result.status {
                        SloStatus::Pass => passed += 1,
                        SloStatus::Fail => failed += 1,
                        SloStatus::Warning => warnings += 1,
                        SloStatus::Exempt => exemptions += 1,
                    }
                    results.push(result);
                }
                Err(e) => {
                    error!("Failed to check SLO {}: {}", slo_id, e);
                    results.push(SloCheckResult {
                        slo_name: slo_id.clone(),
                        status: SloStatus::Fail,
                        target_value: 0.0,
                        actual_value: 0.0,
                        tolerance_applied: false,
                        exemption_applied: None,
                        violation_severity: Some("critical".to_string()),
                        message: format!("Error checking SLO: {}", e),
                    });
                    failed += 1;
                }
            }
        }

        Ok(CheckSummary {
            total_slos: self.config.slos.len() as u32,
            passed,
            failed,
            warnings,
            exemptions,
            environment: self.environment.clone(),
            timestamp: Utc::now(),
            results,
        })
    }

    /// Check an individual SLO
    fn check_individual_slo(
        &self,
        slo_id: &str,
        slo: &Slo,
        metrics: &MetricsData,
    ) -> Result<SloCheckResult> {
        // Check if SLO has exemption
        if let Some(exemption_reason) = self.check_exemptions(slo_id)? {
            return Ok(SloCheckResult {
                slo_name: slo_id.to_string(),
                status: SloStatus::Exempt,
                target_value: 0.0,
                actual_value: 0.0,
                tolerance_applied: false,
                exemption_applied: Some(exemption_reason.clone()),
                violation_severity: None,
                message: format!("SLO exempt: {}", exemption_reason),
            });
        }

        // Get metric data
        let metric_value = metrics.metrics.get(&slo.metric_name)
            .ok_or_else(|| anyhow!("Metric {} not found in metrics data", slo.metric_name))?;

        // Validate minimum samples
        if metric_value.samples < self.config.slo_config.min_samples {
            return Ok(SloCheckResult {
                slo_name: slo_id.to_string(),
                status: SloStatus::Warning,
                target_value: 0.0,
                actual_value: 0.0,
                tolerance_applied: false,
                exemption_applied: None,
                violation_severity: Some("info".to_string()),
                message: format!(
                    "Insufficient samples: {} < {}",
                    metric_value.samples,
                    self.config.slo_config.min_samples
                ),
            });
        }

        // Check P95 target (primary SLO requirement)
        if let Some(p95_target) = slo.targets.p95 {
            let actual_p95 = metric_value.values.get("p95")
                .ok_or_else(|| anyhow!("P95 value not found for metric {}", slo.metric_name))?;

            return self.evaluate_slo_target(
                slo_id,
                "p95",
                p95_target,
                *actual_p95,
                slo,
            );
        }

        // Check other targets if P95 not available
        for (percentile, target) in [
            ("p99", slo.targets.p99),
            ("p50", slo.targets.p50),
            ("max", slo.targets.max),
        ] {
            if let Some(target_value) = target {
                if let Some(actual_value) = metric_value.values.get(percentile) {
                    return self.evaluate_slo_target(
                        slo_id,
                        percentile,
                        target_value,
                        *actual_value,
                        slo,
                    );
                }
            }
        }

        Err(anyhow!("No valid target found for SLO {}", slo_id))
    }

    /// Evaluate a specific SLO target
    fn evaluate_slo_target(
        &self,
        slo_id: &str,
        target_type: &str,
        target_value: f64,
        actual_value: f64,
        slo: &Slo,
    ) -> Result<SloCheckResult> {
        // Apply environment-specific multiplier
        let adjusted_target = self.apply_environment_multiplier(target_value)?;
        
        // Apply tolerance if enabled
        let tolerance_applied = self.tolerance_mode;
        let final_target = if tolerance_applied {
            adjusted_target * (1.0 + self.config.slo_config.tolerance_threshold)
        } else {
            adjusted_target
        };

        // Determine if SLO is violated
        let is_violation = actual_value > final_target;
        
        // Determine severity
        let severity = self.get_violation_severity(slo_id);
        
        // Determine enforcement action
        let status = match self.config.slo_config.enforcement_mode.as_str() {
            "strict" if is_violation => SloStatus::Fail,
            "warn" if is_violation => SloStatus::Warning,
            "disabled" => SloStatus::Pass,
            _ => SloStatus::Pass,
        };

        let message = if is_violation {
            format!(
                "SLO violation: {} {}ms > target {}ms ({})",
                target_type,
                actual_value,
                final_target,
                if tolerance_applied { "with tolerance" } else { "strict" }
            )
        } else {
            format!(
                "SLO met: {} {}ms <= target {}ms",
                target_type,
                actual_value,
                final_target
            )
        };

        Ok(SloCheckResult {
            slo_name: slo_id.to_string(),
            status,
            target_value: final_target,
            actual_value,
            tolerance_applied,
            exemption_applied: None,
            violation_severity: if is_violation { Some(severity) } else { None },
            message,
        })
    }

    /// Apply environment-specific multiplier
    fn apply_environment_multiplier(&self, target: f64) -> Result<f64> {
        if let Some(env_configs) = &self.config.environments {
            if let Some(env_config) = env_configs.get(&self.environment) {
                if let Some(multiplier) = env_config.slo_multiplier {
                    return Ok(target * multiplier);
                }
            }
        }
        Ok(target)
    }

    /// Check for SLO exemptions
    fn check_exemptions(&self, slo_id: &str) -> Result<Option<String>> {
        if let Some(exemptions) = &self.config.exemptions {
            // Check temporary exemptions
            if let Some(temp_exemptions) = &exemptions.temporary {
                for exemption in temp_exemptions {
                    if exemption.slo == slo_id || exemption.slo == "*" {
                        // Check if exemption is still valid
                        let expires = DateTime::parse_from_str(&exemption.expires, "%Y-%m-%d")
                            .context("Invalid exemption expiry date")?;
                        
                        if Utc::now() < expires.with_timezone(&Utc) {
                            return Ok(Some(format!(
                                "Temporary exemption: {} (expires: {})",
                                exemption.reason,
                                exemption.expires
                            )));
                        }
                    }
                }
            }

            // Check permanent exemptions (simplified - would need condition evaluation)
            if let Some(perm_exemptions) = &exemptions.permanent {
                for exemption in perm_exemptions {
                    if exemption.slo == slo_id || exemption.slo == "*" {
                        // For now, just check if it's a startup condition
                        if exemption.condition.contains("system_startup_time") {
                            return Ok(Some(format!(
                                "Permanent exemption: {}",
                                exemption.reason
                            )));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get violation severity for SLO
    fn get_violation_severity(&self, slo_id: &str) -> String {
        if let Some(alerting) = &self.config.alerting {
            if let Some(severity_levels) = &alerting.severity_levels {
                for (level, slos) in severity_levels {
                    if slos.contains(&slo_id.to_string()) {
                        return level.clone();
                    }
                }
            }
        }
        "info".to_string()
    }
}

/// Load SLO configuration from YAML file
fn load_slo_config<P: AsRef<Path>>(path: P) -> Result<SloConfig> {
    let content = fs::read_to_string(path)
        .context("Failed to read SLO configuration file")?;
    
    let config: SloConfig = serde_yaml::from_str(&content)
        .context("Failed to parse SLO configuration YAML")?;
    
    Ok(config)
}

/// Load metrics data from JSON file
fn load_metrics_data<P: AsRef<Path>>(path: P) -> Result<MetricsData> {
    let content = fs::read_to_string(path)
        .context("Failed to read metrics data file")?;
    
    let metrics: MetricsData = serde_json::from_str(&content)
        .context("Failed to parse metrics JSON")?;
    
    Ok(metrics)
}

/// Generate report in specified format
fn generate_report(summary: &CheckSummary, format: &str, output_path: Option<&str>) -> Result<()> {
    let content = match format {
        "json" => serde_json::to_string_pretty(summary)?,
        "html" => generate_html_report(summary)?,
        _ => return Err(anyhow!("Unsupported report format: {}", format)),
    };

    if let Some(path) = output_path {
        fs::write(path, content)?;
        info!("Report written to: {}", path);
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Generate HTML report
fn generate_html_report(summary: &CheckSummary) -> Result<String> {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<title>SLO Check Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
    html.push_str(".pass { color: green; }\n");
    html.push_str(".fail { color: red; }\n");
    html.push_str(".warning { color: orange; }\n");
    html.push_str(".exempt { color: blue; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; }\n");
    html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
    html.push_str("th { background-color: #f2f2f2; }\n");
    html.push_str("</style>\n</head>\n<body>\n");
    
    html.push_str(&format!("<h1>SLO Check Report - {}</h1>\n", summary.environment));
    html.push_str(&format!("<p>Generated: {}</p>\n", summary.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
    
    html.push_str("<h2>Summary</h2>\n");
    html.push_str("<ul>\n");
    html.push_str(&format!("<li>Total SLOs: {}</li>\n", summary.total_slos));
    html.push_str(&format!("<li class=\"pass\">Passed: {}</li>\n", summary.passed));
    html.push_str(&format!("<li class=\"fail\">Failed: {}</li>\n", summary.failed));
    html.push_str(&format!("<li class=\"warning\">Warnings: {}</li>\n", summary.warnings));
    html.push_str(&format!("<li class=\"exempt\">Exemptions: {}</li>\n", summary.exemptions));
    html.push_str("</ul>\n");
    
    html.push_str("<h2>Details</h2>\n");
    html.push_str("<table>\n");
    html.push_str("<tr><th>SLO</th><th>Status</th><th>Target</th><th>Actual</th><th>Message</th></tr>\n");
    
    for result in &summary.results {
        let status_class = match result.status {
            SloStatus::Pass => "pass",
            SloStatus::Fail => "fail",
            SloStatus::Warning => "warning",
            SloStatus::Exempt => "exempt",
        };
        
        html.push_str(&format!(
            "<tr><td>{}</td><td class=\"{}\">({:?})</td><td>{:.2}</td><td>{:.2}</td><td>{}</td></tr>\n",
            result.slo_name,
            status_class,
            result.status,
            result.target_value,
            result.actual_value,
            result.message
        ));
    }
    
    html.push_str("</table>\n");
    html.push_str("</body>\n</html>");
    
    Ok(html)
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let matches = Command::new("check_slo")
        .about("Polymera OS SLO Checker")
        .version("1.0.0")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("SLO configuration file")
                .default_value("perf/slo.yaml"),
        )
        .arg(
            Arg::new("metrics")
                .long("metrics")
                .short('m')
                .value_name("FILE")
                .help("Metrics data file (JSON)")
                .required(true),
        )
        .arg(
            Arg::new("environment")
                .long("env")
                .short('e')
                .value_name("ENV")
                .help("Environment name")
                .default_value("development"),
        )
        .arg(
            Arg::new("tolerance")
                .long("tolerance")
                .help("Apply tolerance threshold")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .value_name("FILE")
                .help("Output file for report"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .value_name("FORMAT")
                .help("Report format")
                .default_value("json")
                .value_parser(["json", "html"]),
        )
        .arg(
            Arg::new("fail-on-violation")
                .long("fail-on-violation")
                .help("Exit with non-zero code on SLO violations")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Parse arguments
    let config_path = matches.get_one::<String>("config").unwrap();
    let metrics_path = matches.get_one::<String>("metrics").unwrap();
    let environment = matches.get_one::<String>("environment").unwrap();
    let tolerance_mode = matches.get_flag("tolerance");
    let output_path = matches.get_one::<String>("output");
    let format = matches.get_one::<String>("format").unwrap();
    let fail_on_violation = matches.get_flag("fail-on-violation");

    info!("Loading SLO configuration from: {}", config_path);
    let config = load_slo_config(config_path)?;

    info!("Loading metrics data from: {}", metrics_path);
    let metrics = load_metrics_data(metrics_path)?;

    info!("Checking SLOs for environment: {}", environment);
    let checker = SloChecker::new(config, environment.clone(), tolerance_mode);
    let summary = checker.check_slos(&metrics)?;

    // Generate report
    generate_report(&summary, format, output_path.map(|s| s.as_str()))?;

    // Print summary to stdout
    println!("SLO Check Summary:");
    println!("  Environment: {}", summary.environment);
    println!("  Total SLOs: {}", summary.total_slos);
    println!("  Passed: {}", summary.passed);
    println!("  Failed: {}", summary.failed);
    println!("  Warnings: {}", summary.warnings);
    println!("  Exemptions: {}", summary.exemptions);

    // Exit with appropriate code
    if fail_on_violation && summary.failed > 0 {
        error!("SLO violations detected, failing build");
        process::exit(1);
    } else if summary.failed > 0 {
        warn!("SLO violations detected but not failing build");
    } else {
        info!("All SLOs passed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_config() -> SloConfig {
        serde_yaml::from_str(r#"
metadata:
  version: "1.0"
  description: "Test SLO config"
  owner: "test"
  updated: "2024-01-01"

slo_config:
  enforcement_mode: "strict"
  tolerance_threshold: 0.05
  min_samples: 5
  measurement_window: 300

slos:
  test_slo:
    name: "Test SLO"
    description: "Test latency"
    metric_name: "test.latency"
    metric_type: "histogram"
    targets:
      p95: 100
    tags:
      - "test"
"#).unwrap()
    }

    fn create_test_metrics(p95_value: f64) -> MetricsData {
        let mut values = HashMap::new();
        values.insert("p95".to_string(), p95_value);
        
        let mut metrics = HashMap::new();
        metrics.insert("test.latency".to_string(), MetricValue {
            metric_type: "histogram".to_string(),
            samples: 100,
            values,
            metadata: None,
        });

        MetricsData {
            timestamp: Utc::now(),
            environment: "test".to_string(),
            metrics,
        }
    }

    #[test]
    fn test_slo_pass() {
        let config = create_test_config();
        let metrics = create_test_metrics(50.0); // Well under 100ms target
        
        let checker = SloChecker::new(config, "test".to_string(), false);
        let summary = checker.check_slos(&metrics).unwrap();
        
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn test_slo_fail() {
        let config = create_test_config();
        let metrics = create_test_metrics(150.0); // Over 100ms target
        
        let checker = SloChecker::new(config, "test".to_string(), false);
        let summary = checker.check_slos(&metrics).unwrap();
        
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn test_tolerance_mode() {
        let config = create_test_config();
        let metrics = create_test_metrics(104.0); // Slightly over 100ms but within 5% tolerance
        
        let checker = SloChecker::new(config, "test".to_string(), true);
        let summary = checker.check_slos(&metrics).unwrap();
        
        // Should pass with tolerance
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn test_insufficient_samples() {
        let config = create_test_config();
        let mut metrics = create_test_metrics(50.0);
        
        // Set samples below minimum
        metrics.metrics.get_mut("test.latency").unwrap().samples = 3;
        
        let checker = SloChecker::new(config, "test".to_string(), false);
        let summary = checker.check_slos(&metrics).unwrap();
        
        assert_eq!(summary.warnings, 1);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 0);
    }
}
