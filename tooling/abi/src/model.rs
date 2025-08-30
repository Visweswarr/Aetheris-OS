use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error types for ABI model validation
#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Duplicate syscall ID: {0}")]
    DuplicateSyscallId(u32),
    
    #[error("Duplicate error code ID: {0}")]
    DuplicateErrorCodeId(u32),
    
    #[error("Duplicate feature ID: {0}")]
    DuplicateFeatureId(u32),
    
    #[error("Invalid syscall ID: {0} (must be > 0)")]
    InvalidSyscallId(u32),
    
    #[error("Invalid error code ID: {0} (must be >= 0)")]
    InvalidErrorCodeId(u32),
    
    #[error("Invalid feature ID: {0} (must be > 0)")]
    InvalidFeatureId(u32),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid enum value: {0}")]
    InvalidEnumValue(String),
    
    #[error("Invalid argument type: {0}")]
    InvalidArgumentType(String),
    
    #[error("Circular dependency detected in features")]
    CircularDependency,
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// Result type for ABI model operations
pub type ModelResult<T> = Result<T, ModelError>;

/// Architecture-specific calling convention information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchConfig {
    pub calling_convention: String,
    pub register_mapping: RegisterMapping,
}

/// Register mapping for different architectures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterMapping {
    pub args: Vec<String>,
    pub return_reg: String,
    pub error_reg: String,
}

/// System call argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallArg {
    pub name: String,
    pub type_name: String,
    pub doc: Option<String>,
    pub len: Option<String>,
    pub check: Option<String>,
    pub max: Option<u64>,
    pub min: Option<u64>,
    pub enum_type: Option<String>,
}

/// System call return value definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallReturn {
    pub type_name: String,
    pub doc: Option<String>,
    pub errno: bool,
}

/// System call safety information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallSafety {
    pub capabilities: Vec<String>,
    pub checks: Vec<String>,
    pub notes: Vec<String>,
}

/// System call definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Syscall {
    pub id: u32,
    pub name: String,
    pub stability: String,
    pub args: Vec<SyscallArg>,
    pub ret: SyscallReturn,
    pub safety: Option<SyscallSafety>,
    pub doc: Option<String>,
    pub category: Option<String>,
}

/// Enum value definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumValue {
    pub name: String,
    pub value: u32,
    pub doc: Option<String>,
}

/// Enum definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub values: Vec<EnumValue>,
}

/// System calls schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallsSchema {
    pub version: u32,
    pub arch: HashMap<String, ArchConfig>,
    pub syscalls: Vec<Syscall>,
    pub enums: Vec<Enum>,
}

/// Error code definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCode {
    pub id: u32,
    pub name: String,
    pub doc: String,
    pub category: String,
    pub stability: String,
}

/// Error codes schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCodesSchema {
    pub version: u32,
    pub errno: Vec<ErrorCode>,
    pub categories: HashMap<String, CategoryInfo>,
}

/// Category information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    pub name: String,
    pub description: String,
}

/// Feature definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: u32,
    pub name: String,
    pub doc: String,
    pub category: String,
    pub stability: String,
    pub introduced: String,
    pub dependencies: Vec<String>,
    pub runtime_check: bool,
}

/// Features schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesSchema {
    pub version: u32,
    pub features: Vec<Feature>,
    pub categories: HashMap<String, CategoryInfo>,
    pub stability_levels: HashMap<String, StabilityLevel>,
    pub feature_groups: HashMap<String, FeatureGroup>,
    pub compatibility: HashMap<String, CompatibilityInfo>,
}

/// Stability level information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityLevel {
    pub name: String,
    pub description: String,
    pub breaking_changes: bool,
    pub deprecation_policy: String,
}

/// Feature group definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureGroup {
    pub name: String,
    pub description: String,
    pub features: Vec<String>,
    pub required: bool,
}

/// Compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub description: String,
    pub features: Vec<String>,
    pub breaking_changes: bool,
}

/// Complete ABI schema combining all components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiSchema {
    pub syscalls: SyscallsSchema,
    pub error_codes: ErrorCodesSchema,
    pub features: FeaturesSchema,
    pub schema_hash: String,
}

impl SyscallsSchema {
    /// Validate the syscalls schema
    pub fn validate(&self) -> ModelResult<()> {
        // Check for duplicate IDs
        let mut ids = std::collections::HashSet::new();
        for syscall in &self.syscalls {
            if !ids.insert(syscall.id) {
                return Err(ModelError::DuplicateSyscallId(syscall.id));
            }
            
            if syscall.id == 0 {
                return Err(ModelError::InvalidSyscallId(syscall.id));
            }
        }
        
        // Validate enum references
        let enum_names: std::collections::HashSet<_> = self.enums.iter().map(|e| &e.name).collect();
        for syscall in &self.syscalls {
            for arg in &syscall.args {
                if let Some(enum_name) = &arg.enum_type {
                    if !enum_names.contains(enum_name) {
                        return Err(ModelError::InvalidEnumValue(format!(
                            "Enum '{}' referenced by syscall '{}' not found",
                            enum_name, syscall.name
                        )));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get syscall by ID
    pub fn get_syscall(&self, id: u32) -> Option<&Syscall> {
        self.syscalls.iter().find(|s| s.id == id)
    }
    
    /// Get syscall by name
    pub fn get_syscall_by_name(&self, name: &str) -> Option<&Syscall> {
        self.syscalls.iter().find(|s| s.name == name)
    }
    
    /// Get all syscalls in a category
    pub fn get_syscalls_by_category(&self, category: &str) -> Vec<&Syscall> {
        self.syscalls.iter()
            .filter(|s| s.category.as_deref() == Some(category))
            .collect()
    }
}

impl ErrorCodesSchema {
    /// Validate the error codes schema
    pub fn validate(&self) -> ModelResult<()> {
        // Check for duplicate IDs
        let mut ids = std::collections::HashSet::new();
        for errno in &self.errno {
            if !ids.insert(errno.id) {
                return Err(ModelError::DuplicateErrorCodeId(errno.id));
            }
        }
        
        // Validate category references
        for errno in &self.errno {
            if !self.categories.contains_key(&errno.category) {
                return Err(ModelError::ValidationFailed(format!(
                    "Category '{}' referenced by error code '{}' not found",
                    errno.category, errno.name
                )));
            }
        }
        
        Ok(())
    }
    
    /// Get error code by ID
    pub fn get_error_code(&self, id: u32) -> Option<&ErrorCode> {
        self.errno.iter().find(|e| e.id == id)
    }
    
    /// Get error code by name
    pub fn get_error_code_by_name(&self, name: &str) -> Option<&ErrorCode> {
        self.errno.iter().find(|e| e.name == name)
    }
    
    /// Get all error codes in a category
    pub fn get_error_codes_by_category(&self, category: &str) -> Vec<&ErrorCode> {
        self.errno.iter()
            .filter(|e| e.category == category)
            .collect()
    }
}

impl FeaturesSchema {
    /// Validate the features schema
    pub fn validate(&self) -> ModelResult<()> {
        // Check for duplicate IDs
        let mut ids = std::collections::HashSet::new();
        for feature in &self.features {
            if !ids.insert(feature.id) {
                return Err(ModelError::DuplicateFeatureId(feature.id));
            }
            
            if feature.id == 0 {
                return Err(ModelError::InvalidFeatureId(feature.id));
            }
        }
        
        // Check for circular dependencies
        if self.has_circular_dependencies() {
            return Err(ModelError::CircularDependency);
        }
        
        // Validate category references
        for feature in &self.features {
            if !self.categories.contains_key(&feature.category) {
                return Err(ModelError::ValidationFailed(format!(
                    "Category '{}' referenced by feature '{}' not found",
                    feature.category, feature.name
                )));
            }
        }
        
        // Validate stability level references
        for feature in &self.features {
            if !self.stability_levels.contains_key(&feature.stability) {
                return Err(ModelError::ValidationFailed(format!(
                    "Stability level '{}' referenced by feature '{}' not found",
                    feature.stability, feature.name
                )));
            }
        }
        
        Ok(())
    }
    
    /// Check for circular dependencies in features
    fn has_circular_dependencies(&self) -> bool {
        use std::collections::{HashMap, HashSet};
        
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut adj_list = HashMap::new();
        
        // Build adjacency list
        for feature in &self.features {
            let deps: Vec<&str> = feature.dependencies.iter().map(|s| s.as_str()).collect();
            adj_list.insert(&feature.name, deps);
        }
        
        // Check for cycles using DFS
        for feature in &self.features {
            if !visited.contains(&feature.name) {
                if self.has_cycle_dfs(&feature.name, &mut visited, &mut rec_stack, &adj_list) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// DFS helper to detect cycles
    fn has_cycle_dfs<'a>(
        &self,
        feature_name: &'a str,
        visited: &mut HashSet<&'a str>,
        rec_stack: &mut HashSet<&'a str>,
        adj_list: &HashMap<&'a str, Vec<&'a str>>,
    ) -> bool {
        visited.insert(feature_name);
        rec_stack.insert(feature_name);
        
        if let Some(deps) = adj_list.get(feature_name) {
            for dep in deps {
                if !visited.contains(dep) {
                    if self.has_cycle_dfs(dep, visited, rec_stack, adj_list) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(feature_name);
        false
    }
    
    /// Get feature by ID
    pub fn get_feature(&self, id: u32) -> Option<&Feature> {
        self.features.iter().find(|f| f.id == id)
    }
    
    /// Get feature by name
    pub fn get_feature_by_name(&self, name: &str) -> Option<&Feature> {
        self.features.iter().find(|f| f.name == name)
    }
    
    /// Get all features in a category
    pub fn get_features_by_category(&self, category: &str) -> Vec<&Feature> {
        self.features.iter()
            .filter(|f| f.category == category)
            .collect()
    }
    
    /// Get all features in a feature group
    pub fn get_features_by_group(&self, group_name: &str) -> Vec<&Feature> {
        if let Some(group) = self.feature_groups.get(group_name) {
            group.features.iter()
                .filter_map(|name| self.get_feature_by_name(name))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Check if a feature is available (all dependencies satisfied)
    pub fn is_feature_available(&self, feature_name: &str, available_features: &HashSet<u32>) -> bool {
        if let Some(feature) = self.get_feature_by_name(feature_name) {
            // Check if the feature itself is available
            if !available_features.contains(&feature.id) {
                return false;
            }
            
            // Check if all dependencies are available
            for dep_name in &feature.dependencies {
                if let Some(dep) = self.get_feature_by_name(dep_name) {
                    if !available_features.contains(&dep.id) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            
            true
        } else {
            false
        }
    }
}

impl AbiSchema {
    /// Validate the complete ABI schema
    pub fn validate(&self) -> ModelResult<()> {
        self.syscalls.validate()?;
        self.error_codes.validate()?;
        self.features.validate()?;
        Ok(())
    }
    
    /// Generate a feature bitset from available feature IDs
    pub fn generate_feature_bitset(&self, available_features: &[u32]) -> u64 {
        let mut bitset = 0u64;
        for &feature_id in available_features {
            if feature_id <= 64 {
                bitset |= 1u64 << (feature_id - 1);
            }
        }
        bitset
    }
    
    /// Parse a feature bitset to get available feature IDs
    pub fn parse_feature_bitset(&self, bitset: u64) -> Vec<u32> {
        let mut features = Vec::new();
        for i in 0..64 {
            if bitset & (1u64 << i) != 0 {
                features.push((i + 1) as u32);
            }
        }
        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_syscalls_validation() {
        let mut schema = SyscallsSchema {
            version: 2,
            arch: HashMap::new(),
            syscalls: vec![],
            enums: vec![],
        };
        
        // Test valid syscall
        schema.syscalls.push(Syscall {
            id: 1,
            name: "test".to_string(),
            stability: "stable".to_string(),
            args: vec![],
            ret: SyscallReturn {
                type_name: "i32".to_string(),
                doc: None,
                errno: true,
            },
            safety: None,
            doc: None,
            category: None,
        });
        
        assert!(schema.validate().is_ok());
        
        // Test duplicate ID
        schema.syscalls.push(Syscall {
            id: 1,
            name: "test2".to_string(),
            stability: "stable".to_string(),
            args: vec![],
            ret: SyscallReturn {
                type_name: "i32".to_string(),
                doc: None,
                errno: true,
            },
            safety: None,
            doc: None,
            category: None,
        });
        
        assert!(schema.validate().is_err());
    }
    
    #[test]
    fn test_features_validation() {
        let mut schema = FeaturesSchema {
            version: 2,
            features: vec![],
            categories: HashMap::new(),
            stability_levels: HashMap::new(),
            feature_groups: HashMap::new(),
            compatibility: HashMap::new(),
        };
        
        // Add required categories and stability levels
        schema.categories.insert("test".to_string(), CategoryInfo {
            name: "Test".to_string(),
            description: "Test category".to_string(),
        });
        
        schema.stability_levels.insert("stable".to_string(), StabilityLevel {
            name: "Stable".to_string(),
            description: "Stable feature".to_string(),
            breaking_changes: false,
            deprecation_policy: "Will not be removed".to_string(),
        });
        
        // Test valid feature
        schema.features.push(Feature {
            id: 1,
            name: "test".to_string(),
            doc: "Test feature".to_string(),
            category: "test".to_string(),
            stability: "stable".to_string(),
            introduced: "v0.2.0".to_string(),
            dependencies: vec![],
            runtime_check: true,
        });
        
        assert!(schema.validate().is_ok());
        
        // Test duplicate ID
        schema.features.push(Feature {
            id: 1,
            name: "test2".to_string(),
            doc: "Test feature 2".to_string(),
            category: "test".to_string(),
            stability: "stable".to_string(),
            introduced: "v0.2.0".to_string(),
            dependencies: vec![],
            runtime_check: true,
        });
        
        assert!(schema.validate().is_err());
    }
    
    #[test]
    fn test_circular_dependencies() {
        let mut schema = FeaturesSchema {
            version: 2,
            features: vec![],
            categories: HashMap::new(),
            stability_levels: HashMap::new(),
            feature_groups: HashMap::new(),
            compatibility: HashMap::new(),
        };
        
        // Add required categories and stability levels
        schema.categories.insert("test".to_string(), CategoryInfo {
            name: "Test".to_string(),
            description: "Test category".to_string(),
        });
        
        schema.stability_levels.insert("stable".to_string(), StabilityLevel {
            name: "Stable".to_string(),
            description: "Stable feature".to_string(),
            breaking_changes: false,
            deprecation_policy: "Will not be removed".to_string(),
        });
        
        // Test circular dependency: A -> B -> A
        schema.features.push(Feature {
            id: 1,
            name: "A".to_string(),
            doc: "Feature A".to_string(),
            category: "test".to_string(),
            stability: "stable".to_string(),
            introduced: "v0.2.0".to_string(),
            dependencies: vec!["B".to_string()],
            runtime_check: true,
        });
        
        schema.features.push(Feature {
            id: 2,
            name: "B".to_string(),
            doc: "Feature B".to_string(),
            category: "test".to_string(),
            stability: "stable".to_string(),
            introduced: "v0.2.0".to_string(),
            dependencies: vec!["A".to_string()],
            runtime_check: true,
        });
        
        assert!(schema.validate().is_err());
    }
}
