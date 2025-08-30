package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"reflect"
)

// FuzzJSONParser tests JSON parsing with malformed input data
func FuzzJSONParser(data []byte) int {
	// Test with UTF-8 string conversion
	jsonStr := string(data)
	
	// Test basic JSON parsing
	var parsed interface{}
	if err := json.Unmarshal(data, &parsed); err != nil {
		// Expected errors for malformed JSON
		return 0
	}
	
	// Test JSON serialization back to bytes
	serialized, err := json.Marshal(parsed)
	if err != nil {
		return 0
	}
	
	// Test with truncated data
	if len(data) > 1 {
		truncated := data[:len(data)-1]
		_ = json.Unmarshal(truncated, &parsed)
	}
	
	// Test with corrupted data
	if len(data) > 10 {
		corrupted := make([]byte, len(data))
		copy(corrupted, data)
		// Flip some bits
		for i := 0; i < 10 && i < len(corrupted); i++ {
			corrupted[i] = corrupted[i] + 1
		}
		_ = json.Unmarshal(corrupted, &parsed)
	}
	
	// Test with nested structures
	if len(data) > 20 {
		complexJSON := fmt.Sprintf(`{
			"string": "%s",
			"number": %d,
			"boolean": %t,
			"null": null,
			"array": [1, 2, 3],
			"object": {"nested": "value"}
		}`, jsonStr[:min(10, len(jsonStr))], len(data), len(data)%2 == 0)
		
		_ = json.Unmarshal([]byte(complexJSON), &parsed)
	}
	
	// Test with very large numbers
	if len(data) > 0 {
		largeNumber := fmt.Sprintf("%d", len(data)*1000000)
		_ = json.Unmarshal([]byte(largeNumber), &parsed)
	}
	
	// Test with escaped strings
	if len(data) > 0 {
		escapedString := fmt.Sprintf(`"%s"`, escapeString(jsonStr))
		_ = json.Unmarshal([]byte(escapedString), &parsed)
	}
	
	// Test with unicode data
	if len(data) > 0 {
		unicodeString := fmt.Sprintf(`"%s"`, toUnicodeEscape(jsonStr))
		_ = json.Unmarshal([]byte(unicodeString), &parsed)
	}
	
	// Test with different JSON formats
	testDifferentFormats(data, jsonStr)
	
	// Test edge cases
	testEdgeCases(data)
	
	// Test with various data types
	testDataTypes(data)
	
	return 1
}

func testDifferentFormats(data []byte, jsonStr string) {
	var parsed interface{}
	
	// Test with single quotes (invalid JSON but test robustness)
	singleQuotes := bytes.ReplaceAll(data, []byte(`"`), []byte(`'`))
	_ = json.Unmarshal(singleQuotes, &parsed)
	
	// Test with Python boolean values (invalid JSON but test robustness)
	pythonBools := bytes.ReplaceAll(data, []byte(`true`), []byte(`True`))
	pythonBools = bytes.ReplaceAll(pythonBools, []byte(`false`), []byte(`False`))
	_ = json.Unmarshal(pythonBools, &parsed)
	
	// Test with Python null (invalid JSON but test robustness)
	pythonNull := bytes.ReplaceAll(data, []byte(`null`), []byte(`None`))
	_ = json.Unmarshal(pythonNull, &parsed)
}

func testEdgeCases(data []byte) {
	var parsed interface{}
	
	// Test with empty data
	_ = json.Unmarshal([]byte(""), &parsed)
	
	// Test with single character
	if len(data) > 0 {
		singleChar := data[:1]
		_ = json.Unmarshal(singleChar, &parsed)
	}
	
	// Test with very large data
	if len(data) > 1024 {
		_ = json.Unmarshal(data, &parsed)
	}
}

func testDataTypes(data []byte) {
	var parsed interface{}
	
	// Test with different target types
	var stringVal string
	_ = json.Unmarshal(data, &stringVal)
	
	var intVal int
	_ = json.Unmarshal(data, &intVal)
	
	var floatVal float64
	_ = json.Unmarshal(data, &floatVal)
	
	var boolVal bool
	_ = json.Unmarshal(data, &boolVal)
	
	var mapVal map[string]interface{}
	_ = json.Unmarshal(data, &mapVal)
	
	var sliceVal []interface{}
	_ = json.Unmarshal(data, &sliceVal)
	
	// Test with struct
	type TestStruct struct {
		StringField string                 `json:"string_field"`
		IntField    int                    `json:"int_field"`
		FloatField  float64                `json:"float_field"`
		BoolField   bool                   `json:"bool_field"`
		MapField    map[string]interface{} `json:"map_field"`
		SliceField  []interface{}          `json:"slice_field"`
	}
	
	var structVal TestStruct
	_ = json.Unmarshal(data, &structVal)
}

func escapeString(s string) string {
	escaped := bytes.ReplaceAll([]byte(s), []byte(`\`), []byte(`\\`))
	escaped = bytes.ReplaceAll(escaped, []byte(`"`), []byte(`\"`))
	return string(escaped)
}

func toUnicodeEscape(s string) string {
	var result bytes.Buffer
	for _, r := range s {
		if r < 128 {
			result.WriteRune(r)
		} else {
			fmt.Fprintf(&result, "\\u%04x", r)
		}
	}
	return result.String()
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
