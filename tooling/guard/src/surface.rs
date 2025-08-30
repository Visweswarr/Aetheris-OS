use crate::diff::generate_diff;
use crate::normalize::normalize_content;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use tokio::fs as tokio_fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceLock {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub surfaces: Vec<Surface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Surface {
    pub id: String,
    pub paths: Vec<String>,
    pub kind: SurfaceKind,
    pub owner: String,
    pub notes: String,
    pub golden_hash: String,
    pub generator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurfaceKind {
    #[serde(rename = "generated")]
    Generated,
    #[serde(rename = "static")]
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceStatus {
    pub status: StatusType,
    pub current_hash: Option<String>,
    pub diff_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusType {
    Ok,
    Drift,
    Error { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allowlist {
    pub allowances: Vec<Allowance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allowance {
    pub path_glob: String,
    pub reason: String,
    pub author: String,
    pub expires: DateTime<Utc>,
}

impl SurfaceLock {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }
    
    pub async fn regenerate_surfaces(&self) -> Result<()> {
        for surface in &self.surfaces {
            if let Some(generator) = &surface.generator {
                info!("🔄 Running generator: {}", generator);
                
                let output = Command::new("bazel")
                    .args(&["run", generator])
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow!(
                        "Generator {} failed: {}",
                        generator,
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Surface {
    pub async fn check_status(&self) -> Result<SurfaceStatus> {
        if let Some(generator) = &self.generator {
            info!("🔄 Running generator: {}", generator);
            
            let output = Command::new("bazel")
                .args(&["run", generator])
                .output()?;
            
            if !output.status.success() {
                return Ok(SurfaceStatus {
                    status: StatusType::Error {
                        error: format!(
                            "Generator {} failed: {}",
                            generator,
                            String::from_utf8_lossy(&output.stderr)
                        ),
                    },
                    current_hash: None,
                    diff_hint: None,
                });
            }
        }
        
        let mut combined_content = String::new();
        let mut all_files_exist = true;
        
        for path in &self.paths {
            match tokio_fs::read_to_string(path).await {
                Ok(content) => {
                    let normalized = normalize_content(&content);
                    combined_content.push_str(&normalized);
                    combined_content.push('\n');
                }
                Err(_) => {
                    all_files_exist = false;
                    break;
                }
            }
        }
        
        if !all_files_exist {
            return Ok(SurfaceStatus {
                status: StatusType::Error {
                    error: "One or more files do not exist".to_string(),
                },
                current_hash: None,
                diff_hint: None,
            });
        }
        
        let current_hash = sha256_hash(&combined_content);
        
        if current_hash == self.golden_hash {
            Ok(SurfaceStatus {
                status: StatusType::Ok,
                current_hash: Some(current_hash),
                diff_hint: None,
            })
        } else {
            let diff_hint = generate_diff(&self.paths, &self.golden_hash, &current_hash).await?;
            
            Ok(SurfaceStatus {
                status: StatusType::Drift,
                current_hash: Some(current_hash),
                diff_hint: Some(diff_hint),
            })
        }
    }
}

impl Allowlist {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Allowlist { allowances: Vec::new() });
        }
        
        let content = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&content)?)
    }
    
    pub fn find_matching_allowance(&self, paths: &[String]) -> Option<&Allowance> {
        for allowance in &self.allowances {
            for path in paths {
                if glob::Pattern::new(&allowance.path_glob)
                    .unwrap()
                    .matches(path)
                {
                    if allowance.expires > Utc::now() {
                        return Some(allowance);
                    }
                }
            }
        }
        None
    }
}

pub fn generate_html_report(
    results: &[(Surface, SurfaceStatus)],
    allowlist: &Allowlist,
    output_path: &Path,
) -> Result<()> {
    let mut html = String::new();
    
    html.push_str(r#"<!DOCTYPE html>
<html>
<head>
    <title>Polymera OS Surface Guard Report</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; }
        .header { background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
        .surface { border: 1px solid #e1e4e8; border-radius: 6px; margin: 10px 0; padding: 15px; }
        .ok { border-left: 4px solid #28a745; }
        .drift { border-left: 4px solid #dc3545; }
        .error { border-left: 4px solid #ffc107; }
        .status { font-weight: bold; padding: 4px 8px; border-radius: 4px; }
        .status-ok { background: #d4edda; color: #155724; }
        .status-drift { background: #f8d7da; color: #721c24; }
        .status-error { background: #fff3cd; color: #856404; }
        .hash { font-family: monospace; font-size: 12px; background: #f6f8fa; padding: 4px; border-radius: 4px; }
        .diff { font-family: monospace; background: #f6f8fa; padding: 10px; border-radius: 4px; margin-top: 10px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🔒 Polymera OS Surface Guard Report</h1>
        <p>Generated at: "#);
    
    html.push_str(&chrono::Utc::now().to_rfc3339());
    html.push_str(r#"</p>
    </div>
"#);
    
    for (surface, status) in results {
        let status_class = match status.status {
            StatusType::Ok => "ok",
            StatusType::Drift => "drift",
            StatusType::Error { .. } => "error",
        };
        
        let status_text = match status.status {
            StatusType::Ok => "OK",
            StatusType::Drift => "DRIFT",
            StatusType::Error { .. } => "ERROR",
        };
        
        let status_style = match status.status {
            StatusType::Ok => "status-ok",
            StatusType::Drift => "status-drift",
            StatusType::Error { .. } => "status-error",
        };
        
        html.push_str(&format!(
            r#"<div class="surface {}">
    <h3>{}</h3>
    <p><strong>Owner:</strong> {}</p>
    <p><strong>Paths:</strong> {}</p>
    <p><strong>Status:</strong> <span class="status {}">{}</span></p>
    <p><strong>Golden Hash:</strong> <span class="hash">{}</span></p>"#,
            status_class, surface.id, surface.owner, surface.paths.join(", "), status_style, status_text, surface.golden_hash
        ));
        
        if let Some(current_hash) = &status.current_hash {
            html.push_str(&format!(
                r#"<p><strong>Current Hash:</strong> <span class="hash">{}</span></p>"#,
                current_hash
            ));
        }
        
        if let Some(diff_hint) = &status.diff_hint {
            html.push_str(&format!(
                r#"<div class="diff"><strong>Diff Hint:</strong><br><pre>{}</pre></div>"#,
                diff_hint
            ));
        }
        
        if let StatusType::Drift = status.status {
            if let Some(allowed) = allowlist.find_matching_allowance(&surface.paths) {
                html.push_str(&format!(
                    r#"<p><strong>✅ Allowed:</strong> {} (by {}) - expires {}</p>"#,
                    allowed.reason, allowed.author, allowed.expires.format("%Y-%m-%d %H:%M:%S UTC")
                ));
            } else {
                html.push_str(r#"<p><strong>❌ Not Allowed:</strong> Requires approval in SURFACE.allow.yaml</p>"#);
            }
        }
        
        html.push_str("</div>\n");
    }
    
    html.push_str(r#"
</body>
</html>"#);
    
    fs::write(output_path, html)?;
    Ok(())
}

fn sha256_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

use log::info;
