use std::collections::HashMap;

pub fn normalize_content(content: &str) -> String {
    let mut normalized = content.to_string();
    
    normalized = normalize_line_endings(normalized);
    normalized = normalize_whitespace(normalized);
    normalized = strip_timestamps(normalized);
    normalized = normalize_markdown_tables(normalized);
    
    normalized
}

fn normalize_line_endings(content: String) -> String {
    content.replace("\r\n", "\n").replace("\r", "\n")
}

fn normalize_whitespace(content: String) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut normalized_lines = Vec::new();
    
    for line in lines {
        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            normalized_lines.push(trimmed);
        }
    }
    
    normalized_lines.join("\n")
}

fn strip_timestamps(content: String) -> String {
    let mut lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        
        if line.contains("Generated at:") || 
           line.contains("Last updated:") ||
           line.contains("Build time:") ||
           line.contains("Timestamp:") ||
           line.contains("Date:") {
            lines.remove(i);
        } else if line.contains("<!--") && line.contains("-->") {
            if line.contains("timestamp") || line.contains("generated") {
                lines.remove(i);
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    
    lines.join("\n")
}

fn normalize_markdown_tables(content: String) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut normalized_lines = Vec::new();
    let mut in_table = false;
    let mut table_lines = Vec::new();
    
    for line in lines {
        if line.starts_with('|') && line.ends_with('|') {
            if !in_table {
                in_table = true;
            }
            table_lines.push(line);
        } else if in_table {
            if line.trim().is_empty() || !line.starts_with('|') {
                in_table = false;
                normalized_lines.extend(normalize_table(&table_lines));
                table_lines.clear();
            }
            if !line.trim().is_empty() {
                normalized_lines.push(line);
            }
        } else {
            normalized_lines.push(line);
        }
    }
    
    if in_table {
        normalized_lines.extend(normalize_table(&table_lines));
    }
    
    normalized_lines.join("\n")
}

fn normalize_table(table_lines: &[&str]) -> Vec<String> {
    if table_lines.len() < 2 {
        return table_lines.iter().map(|&s| s.to_string()).collect();
    }
    
    let mut normalized = Vec::new();
    
    for (i, line) in table_lines.iter().enumerate() {
        if i == 1 {
            continue;
        }
        
        let cells: Vec<&str> = line.split('|').collect();
        let mut normalized_cells = Vec::new();
        
        for cell in cells {
            let trimmed = cell.trim();
            if !trimmed.is_empty() {
                normalized_cells.push(trimmed);
            }
        }
        
        if !normalized_cells.is_empty() {
            normalized.push(format!("| {} |", normalized_cells.join(" | ")));
        }
    }
    
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_line_endings() {
        let input = "line1\r\nline2\rline3\nline4";
        let expected = "line1\nline2\nline3\nline4";
        assert_eq!(normalize_line_endings(input.to_string()), expected);
    }
    
    #[test]
    fn test_normalize_whitespace() {
        let input = "line1  \n  line2  \n\nline3";
        let expected = "line1\nline2\nline3";
        assert_eq!(normalize_whitespace(input.to_string()), expected);
    }
    
    #[test]
    fn test_strip_timestamps() {
        let input = "content1\nGenerated at: 2024-01-01\ncontent2\nLast updated: 2024-01-02";
        let expected = "content1\ncontent2";
        assert_eq!(strip_timestamps(input.to_string()), expected);
    }
    
    #[test]
    fn test_normalize_markdown_tables() {
        let input = "before\n| col1 | col2 |\n|------|------|\n| val1 | val2 |\nafter";
        let expected = "before\n| col1 | col2 |\n| val1 | val2 |\nafter";
        assert_eq!(normalize_markdown_tables(input.to_string()), expected);
    }
}
