use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn generate_diff(
    paths: &[String],
    golden_hash: &str,
    current_hash: &str,
) -> Result<String> {
    let mut diff_output = String::new();
    
    diff_output.push_str(&format!("Hash changed from {} to {}\n", golden_hash, current_hash));
    diff_output.push_str("Files that may have changed:\n");
    
    for path in paths {
        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines().collect();
            let line_count = lines.len();
            
            diff_output.push_str(&format!("\n--- {}\n", path));
            
            if line_count <= 50 {
                for (i, line) in lines.iter().enumerate() {
                    diff_output.push_str(&format!("{:4} {}", i + 1, line));
                    diff_output.push('\n');
                }
            } else {
                for (i, line) in lines.iter().take(25).enumerate() {
                    diff_output.push_str(&format!("{:4} {}", i + 1, line));
                    diff_output.push('\n');
                }
                
                diff_output.push_str(&format!("     ... ({} lines omitted) ...\n", line_count - 50));
                
                for (i, line) in lines.iter().skip(line_count - 25).enumerate() {
                    let actual_line_num = line_count - 25 + i + 1;
                    diff_output.push_str(&format!("{:4} {}", actual_line_num, line));
                    diff_output.push('\n');
                }
            }
        } else {
            diff_output.push_str(&format!("\n--- {} (file not found or unreadable)\n", path));
        }
    }
    
    Ok(diff_output)
}

pub fn generate_unified_diff(
    old_content: &str,
    new_content: &str,
    context_lines: usize,
) -> String {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();
    
    let mut diff_output = String::new();
    diff_output.push_str("--- old\n");
    diff_output.push_str("+++ new\n");
    
    let mut i = 0;
    let mut j = 0;
    
    while i < old_lines.len() || j < new_lines.len() {
        let start_i = i.saturating_sub(context_lines);
        let start_j = j.saturating_sub(context_lines);
        
        let mut has_changes = false;
        let mut change_start = 0;
        
        while i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
            i += 1;
            j += 1;
        }
        
        if i < old_lines.len() || j < new_lines.len() {
            has_changes = true;
            change_start = i;
        }
        
        if has_changes {
            let end_i = (i + context_lines).min(old_lines.len());
            let end_j = (j + context_lines).min(new_lines.len());
            
            diff_output.push_str(&format!("@@ -{},{} +{},{} @@\n", 
                start_i + 1, end_i - start_i, start_j + 1, end_j - start_j));
            
            for k in start_i..end_i {
                if k < old_lines.len() {
                    diff_output.push_str(&format!("-{}", old_lines[k]));
                    diff_output.push('\n');
                }
            }
            
            for k in start_j..end_j {
                if k < new_lines.len() {
                    diff_output.push_str(&format!("+{}", new_lines[k]));
                    diff_output.push('\n');
                }
            }
            
            diff_output.push('\n');
        }
    }
    
    diff_output
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_unified_diff() {
        let old_content = "line1\nline2\nline3\nline4";
        let new_content = "line1\nline2\nmodified\nline4";
        
        let diff = generate_unified_diff(old_content, new_content, 1);
        
        assert!(diff.contains("@@"));
        assert!(diff.contains("-line3"));
        assert!(diff.contains("+modified"));
    }
}
