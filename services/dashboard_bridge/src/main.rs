//! Dashboard Bridge - converts QEMU serial kernel telemetry into dashboard JS.
//!
//! Kernel Telemetry Bridge v1 is intentionally simple: the kernel emits a
//! structured serial line, and this host-side bridge writes the newest captured
//! line into `ui/dashboard/kernel_metrics.js`. No shared memory ABI is claimed.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const METRIC_PREFIX: &str = "POLYMERA_KERNEL_METRIC_V1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelMetrics {
    pub kernel_ticks: u64,
    pub kernel_active_processes: u64,
    pub kernel_syscalls_total: u64,
    pub kernel_ipc_total: u64,
    pub kernel_context_switches_total: u64,
    pub kernel_security_checks_total: u64,
    pub source: String,
    pub updated_at_unix: u64,
}

impl KernelMetrics {
    pub fn unavailable() -> Self {
        Self {
            kernel_ticks: 0,
            kernel_active_processes: 0,
            kernel_syscalls_total: 0,
            kernel_ipc_total: 0,
            kernel_context_switches_total: 0,
            kernel_security_checks_total: 0,
            source: "unavailable".to_string(),
            updated_at_unix: unix_now(),
        }
    }

    pub fn is_real(&self) -> bool {
        self.source == "qemu-serial"
    }

    pub fn write_to_js(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let js = format!("window.POLYMERA_KERNEL_METRICS = {};\n", json);
        std::fs::write(path, js)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BridgeConfig {
    output: PathBuf,
    serial_log: PathBuf,
    once: bool,
    require_real: bool,
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}

fn default_output_path() -> PathBuf {
    repo_root().join("ui/dashboard/kernel_metrics.js")
}

fn default_serial_log_path() -> PathBuf {
    repo_root().join("build_out/qemu-serial.log")
}

fn parse_args(args: &[String]) -> Result<BridgeConfig, String> {
    let mut output: Option<PathBuf> = None;
    let mut serial_log: Option<PathBuf> = None;
    let mut once = false;
    let mut require_real = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--once" => once = true,
            "--require-real" => require_real = true,
            "--serial-log" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "--serial-log requires a path".to_string())?;
                serial_log = Some(PathBuf::from(value));
            }
            "--output" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "--output requires a path".to_string())?;
                output = Some(PathBuf::from(value));
            }
            "-h" | "--help" => return Err(help_text()),
            arg if arg.starts_with('-') => {
                return Err(format!("unknown argument: {arg}\n\n{}", help_text()));
            }
            positional => {
                if output.is_some() {
                    return Err(format!("unexpected extra positional argument: {positional}"));
                }
                output = Some(PathBuf::from(positional));
            }
        }
        i += 1;
    }

    Ok(BridgeConfig {
        output: output.unwrap_or_else(default_output_path),
        serial_log: serial_log.unwrap_or_else(default_serial_log_path),
        once,
        require_real,
    })
}

fn help_text() -> String {
    "usage: dashboard_bridge [--once] [--require-real] [--serial-log <path>] [--output <path>] [output]".to_string()
}

pub fn parse_metric_line(line: &str) -> Option<KernelMetrics> {
    let prefix_start = line.find(METRIC_PREFIX)?;
    let metric_text = &line[prefix_start + METRIC_PREFIX.len()..];
    let mut values = BTreeMap::new();

    for part in metric_text.split_whitespace() {
        let (key, value) = part.split_once('=')?;
        let parsed = value.parse::<u64>().ok()?;
        values.insert(key, parsed);
    }

    Some(KernelMetrics {
        kernel_ticks: *values.get("ticks")?,
        kernel_active_processes: *values.get("active_processes")?,
        kernel_syscalls_total: *values.get("syscalls")?,
        kernel_ipc_total: *values.get("ipc")?,
        kernel_context_switches_total: *values.get("ctx_sw")?,
        kernel_security_checks_total: *values.get("security_checks")?,
        source: "qemu-serial".to_string(),
        updated_at_unix: unix_now(),
    })
}

pub fn parse_latest_metric(serial_text: &str) -> Option<KernelMetrics> {
    serial_text.lines().filter_map(parse_metric_line).last()
}

pub fn collect_from_serial_log(path: &Path) -> std::io::Result<KernelMetrics> {
    let text = std::fs::read_to_string(path)?;
    Ok(parse_latest_metric(&text).unwrap_or_else(KernelMetrics::unavailable))
}

fn collect_or_unavailable(path: &Path) -> KernelMetrics {
    collect_from_serial_log(path).unwrap_or_else(|_| KernelMetrics::unavailable())
}

fn write_once(config: &BridgeConfig) -> Result<KernelMetrics, Box<dyn std::error::Error>> {
    let metrics = collect_or_unavailable(&config.serial_log);
    metrics.write_to_js(&config.output)?;

    println!("[dashboard-bridge] Source: {}", metrics.source);
    println!(
        "[dashboard-bridge] ticks={} active_processes={} syscalls={} ipc={} ctx_sw={}",
        metrics.kernel_ticks,
        metrics.kernel_active_processes,
        metrics.kernel_syscalls_total,
        metrics.kernel_ipc_total,
        metrics.kernel_context_switches_total
    );

    if config.require_real && !metrics.is_real() {
        return Err(format!(
            "no {METRIC_PREFIX} line found in {}",
            config.serial_log.display()
        )
        .into());
    }

    Ok(metrics)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let config = parse_args(&args).map_err(|e| {
        eprintln!("{e}");
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    println!(
        "[dashboard-bridge] Reading kernel serial metrics from {}",
        config.serial_log.display()
    );
    println!(
        "[dashboard-bridge] Writing dashboard metrics to {}",
        config.output.display()
    );

    if config.once {
        write_once(&config)?;
        return Ok(());
    }

    loop {
        write_once(&config)?;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(ticks: u64) -> String {
        format!(
            "POLYMERA_KERNEL_METRIC_V1 ticks={ticks} active_processes=10 syscalls=2 ipc=3 ctx_sw=4 security_checks=5"
        )
    }

    #[test]
    fn parses_valid_metric_line() {
        let m = parse_metric_line(&line(42)).unwrap();
        assert_eq!(m.kernel_ticks, 42);
        assert_eq!(m.kernel_active_processes, 10);
        assert_eq!(m.kernel_syscalls_total, 2);
        assert_eq!(m.kernel_ipc_total, 3);
        assert_eq!(m.kernel_context_switches_total, 4);
        assert_eq!(m.kernel_security_checks_total, 5);
        assert_eq!(m.source, "qemu-serial");
    }

    #[test]
    fn chooses_newest_metric_line() {
        let text = format!("{}\nnoise\n{}\n", line(1), line(99));
        let m = parse_latest_metric(&text).unwrap();
        assert_eq!(m.kernel_ticks, 99);
    }

    #[test]
    fn missing_metric_becomes_unavailable() {
        let m = parse_latest_metric("POLYMERA_MAIN_LOOP_READY\n").unwrap_or_else(KernelMetrics::unavailable);
        assert_eq!(m.source, "unavailable");
        assert!(!m.is_real());
    }

    #[test]
    fn write_creates_valid_js() {
        let m = parse_metric_line(&line(7)).unwrap();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        m.write_to_js(tmp.path()).unwrap();
        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(contents.starts_with("window.POLYMERA_KERNEL_METRICS"));
        assert!(contents.contains("kernel_ticks"));
        assert!(contents.contains("\"source\": \"qemu-serial\""));
    }

    #[test]
    fn require_real_rejects_missing_metrics() {
        let serial = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(serial.path(), "POLYMERA_MAIN_LOOP_READY\n").unwrap();
        let output = tempfile::NamedTempFile::new().unwrap();
        let config = BridgeConfig {
            output: output.path().to_path_buf(),
            serial_log: serial.path().to_path_buf(),
            once: true,
            require_real: true,
        };

        let err = write_once(&config).unwrap_err().to_string();
        assert!(err.contains(METRIC_PREFIX));

        let contents = std::fs::read_to_string(output.path()).unwrap();
        assert!(contents.contains("\"source\": \"unavailable\""));
    }
}
