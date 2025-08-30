use colored::*;
use std::path::PathBuf;
use std::process::Command;

/// Check if a command is available in PATH
pub fn command_available(command: &str) -> bool {
    Command::new(command).arg("--version").output().is_ok()
}

/// Get command version
pub fn get_command_version(command: &str) -> Option<String> {
    Command::new(command)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|version| version.lines().next().unwrap_or("").trim().to_string())
}

/// Check if running in a CI environment
pub fn is_ci_environment() -> bool {
    std::env::var("CI").is_ok() || 
    std::env::var("GITHUB_ACTIONS").is_ok() ||
    std::env::var("GITLAB_CI").is_ok() ||
    std::env::var("JENKINS_URL").is_ok()
}

/// Get project root directory
pub fn get_project_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    
    loop {
        if current.join("Cargo.toml").exists() || 
           current.join("flake.nix").exists() ||
           current.join("WORKSPACE").exists() {
            return Some(current);
        }
        
        if !current.pop() {
            break;
        }
    }
    
    None
}

/// Format file size in human readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human readable format
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    
    if secs == 0 {
        format!("{}ms", millis)
    } else if secs < 60 {
        format!("{}.{}s", secs, millis / 100)
    } else if secs < 3600 {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}m {}s", mins, secs)
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{}h {}m", hours, mins)
    }
}

/// Print progress bar
pub fn print_progress_bar(current: usize, total: usize, width: usize) {
    let percentage = if total > 0 { (current * 100) / total } else { 0 };
    let filled = (width * current) / total;
    let empty = width - filled;
    
    let bar = "█".repeat(filled) + "░".repeat(empty);
    print!("\r[{}] {}%", bar, percentage);
    std::io::stdout().flush().ok();
}

/// Print success message
pub fn print_success(message: &str) {
    println!("{} {}", "✅".green(), message);
}

/// Print error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "❌".red(), message);
}

/// Print warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠️".yellow(), message);
}

/// Print info message
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ️".blue(), message);
}

/// Print step message
pub fn print_step(step: &str) {
    println!("{}", format!("→ {}", step).cyan());
}

/// Print subsection
pub fn print_subsection(title: &str) {
    println!("{}", format!("  {}", title).bold());
}

/// Print detail
pub fn print_detail(key: &str, value: &str) {
    println!("    {}: {}", key.blue(), value);
}

/// Check if terminal supports colors
pub fn supports_colors() -> bool {
    if let Ok(term) = std::env::var("TERM") {
        term != "dumb" && term != "unknown"
    } else {
        false
    }
}

/// Get system information
pub fn get_system_info() -> std::collections::HashMap<String, String> {
    let mut info = std::collections::HashMap::new();
    
    // OS info
    if let Ok(os) = std::env::var("OS") {
        info.insert("os".to_string(), os);
    }
    
    // Architecture
    if let Ok(arch) = std::env::var("PROCESSOR_ARCHITECTURE") {
        info.insert("arch".to_string(), arch);
    }
    
    // Rust version
    if let Ok(rustc) = Command::new("rustc").arg("--version").output() {
        if rustc.status.success() {
            if let Ok(version) = String::from_utf8(rustc.stdout) {
                info.insert("rust_version".to_string(), version.trim().to_string());
            }
        }
    }
    
    // Cargo version
    if let Ok(cargo) = Command::new("cargo").arg("--version").output() {
        if cargo.status.success() {
            if let Ok(version) = String::from_utf8(cargo.stdout) {
                info.insert("cargo_version".to_string(), version.trim().to_string());
            }
        }
    }
    
    info
}

/// Create temporary directory
pub fn create_temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir();
    let dir_name = format!("{}_{}", prefix, chrono::Utc::now().timestamp());
    let temp_path = temp_dir.join(dir_name);
    
    std::fs::create_dir_all(&temp_path)?;
    Ok(temp_path)
}

/// Clean up temporary directory
pub fn cleanup_temp_dir(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// Check if path is in git repository
pub fn is_git_repository(path: &PathBuf) -> bool {
    let mut git_path = path.clone();
    loop {
        if git_path.join(".git").exists() {
            return true;
        }
        if !git_path.pop() {
            break;
        }
    }
    false
}

/// Get git branch name
pub fn get_git_branch(path: &PathBuf) -> Option<String> {
    if !is_git_repository(path) {
        return None;
    }
    
    Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(path)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|branch| branch.trim().to_string())
}

/// Get git commit hash
pub fn get_git_commit(path: &PathBuf) -> Option<String> {
    if !is_git_repository(path) {
        return None;
    }
    
    Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(path)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|commit| commit.trim().to_string())
}

/// Check if file is binary
pub fn is_binary_file(path: &PathBuf) -> bool {
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut buffer = [0; 1024];
        if let Ok(n) = file.read(&mut buffer) {
            // Check for null bytes in the first 1KB
            buffer[..n].iter().any(|&b| b == 0)
        } else {
            false
        }
    } else {
        false
    }
}

/// Get file mime type
pub fn get_mime_type(path: &PathBuf) -> String {
    if let Some(ext) = path.extension() {
        match ext.to_str().unwrap_or("").to_lowercase().as_str() {
            "rs" => "text/x-rust",
            "toml" => "text/x-toml",
            "json" => "application/json",
            "md" => "text/markdown",
            "txt" => "text/plain",
            "exe" | "bin" => "application/octet-stream",
            "so" | "dll" | "dylib" => "application/x-sharedlib",
            "img" | "iso" => "application/octet-stream",
            "qcow2" | "vmdk" => "application/octet-stream",
            _ => "application/octet-stream",
        }
    } else {
        "application/octet-stream".to_string()
    }
}

/// Validate file path
pub fn validate_file_path(path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path_buf = PathBuf::from(path);
    
    // Check if path is absolute
    if !path_buf.is_absolute() {
        // Make it absolute relative to current directory
        let current_dir = std::env::current_dir()?;
        Ok(current_dir.join(path_buf))
    } else {
        Ok(path_buf)
    }
}

/// Ensure directory exists
pub fn ensure_directory(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Copy file with progress
pub fn copy_file_with_progress(src: &PathBuf, dst: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
    let mut src_file = std::fs::File::open(src)?;
    let mut dst_file = std::fs::File::create(dst)?;
    
    let metadata = src_file.metadata()?;
    let file_size = metadata.len();
    let mut buffer = [0; 8192];
    let mut total_copied = 0;
    
    loop {
        let n = src_file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        
        dst_file.write_all(&buffer[..n])?;
        total_copied += n as u64;
        
        // Print progress every 1MB
        if total_copied % (1024 * 1024) == 0 {
            let percentage = (total_copied * 100) / file_size;
            print_progress_bar(total_copied as usize, file_size as usize, 50);
        }
    }
    
    // Clear progress bar
    println!();
    
    Ok(total_copied)
}
