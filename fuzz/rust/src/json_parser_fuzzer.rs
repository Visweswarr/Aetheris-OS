#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::{Value, from_str, from_slice, from_reader};
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Test JSON string parsing
    if let Ok(json_str) = std::str::from_utf8(data) {
        let _ = from_str::<Value>(json_str);
    }

    // Test JSON slice parsing
    let _ = from_slice::<Value>(data);

    // Test JSON reader parsing
    let cursor = Cursor::new(data);
    let _ = from_reader::<Value, Cursor<&[u8]>>(cursor);

    // Test with truncated data
    if data.len() > 1 {
        let truncated = &data[..data.len() - 1];
        if let Ok(json_str) = std::str::from_utf8(truncated) {
            let _ = from_str::<Value>(json_str);
        }
        let _ = from_slice::<Value>(truncated);
    }

    // Test with corrupted data
    if data.len() > 10 {
        let mut corrupted = data.to_vec();
        // Flip some bits
        for i in 0..std::cmp::min(10, data.len()) {
            corrupted[i] = corrupted[i].wrapping_add(1);
        }
        
        if let Ok(json_str) = std::str::from_utf8(&corrupted) {
            let _ = from_str::<Value>(json_str);
        }
        let _ = from_slice::<Value>(&corrupted);
    }

    // Test with nested structures
    if data.len() > 20 {
        // Create a complex JSON structure
        let complex_json = format!(
            r#"{{
                "string": "{}",
                "number": {},
                "boolean": {},
                "null": null,
                "array": [1, 2, 3],
                "object": {{"nested": "value"}}
            }}"#,
            std::str::from_utf8(data).unwrap_or("corrupted"),
            data.len(),
            data.len() % 2 == 0
        );
        
        let _ = from_str::<Value>(&complex_json);
    }

    // Test with very large numbers
    if data.len() > 0 {
        let large_number = format!("{}", data.len() * 1000000);
        let _ = from_str::<Value>(&large_number);
    }

    // Test with escaped strings
    if data.len() > 0 {
        let escaped_string = format!(
            r#""{}""#,
            std::str::from_utf8(data)
                .unwrap_or("corrupted")
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
        );
        let _ = from_str::<Value>(&escaped_string);
    }

    // Test with unicode data
    if data.len() > 0 {
        let unicode_string = format!(
            r#""{}""#,
            std::str::from_utf8(data)
                .unwrap_or("corrupted")
                .chars()
                .map(|c| format!("\\u{:04x}", c as u32))
                .collect::<String>()
        );
        let _ = from_str::<Value>(&unicode_string);
    }
});
