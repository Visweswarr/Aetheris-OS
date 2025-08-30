use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tera::{Context as TeraContext, Tera};

#[derive(Debug, Deserialize)]
pub struct IntentSchema {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub lanes: Vec<String>,
    pub structs: HashMap<String, IntentStruct>,
    pub enums: HashMap<String, IntentEnum>,
    pub limits: IntentLimits,
    pub policy: IntentPolicy,
    pub mapping: IntentMapping,
    pub syscalls: Vec<IntentSyscall>,
    pub audit_codes: Vec<IntentAuditCode>,
}

#[derive(Debug, Deserialize)]
pub struct IntentStruct {
    pub description: String,
    pub fields: Vec<IntentField>,
}

#[derive(Debug, Deserialize)]
pub struct IntentField {
    pub name: String,
    pub r#type: String,
    #[serde(default)]
    pub const_value: Option<String>,
    #[serde(default)]
    pub enum_type: Option<String>,
    #[serde(default)]
    pub len: Option<String>,
    pub doc: String,
}

#[derive(Debug, Deserialize)]
pub struct IntentEnum {
    pub description: String,
    pub values: HashMap<String, IntentEnumValue>,
}

#[derive(Debug, Deserialize)]
pub struct IntentEnumValue {
    pub name: String,
    pub doc: String,
}

#[derive(Debug, Deserialize)]
pub struct IntentLimits {
    pub text_max: u32,
    pub plan_max: u32,
    pub preview_max: u32,
    pub whylog_max: u32,
    pub queue_rt_max: u32,
    pub queue_high_max: u32,
    pub queue_best_max: u32,
}

#[derive(Debug, Deserialize)]
pub struct IntentPolicy {
    pub mode_feature: String,
    pub decisions: Vec<String>,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct IntentMapping {
    pub scopes_to_caps: IntentScopesToCaps,
}

#[derive(Debug, Deserialize)]
pub struct IntentScopesToCaps {
    pub description: String,
    pub mappings: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct IntentSyscall {
    pub id: String,
    pub name: String,
    pub stability: String,
    pub args: Vec<IntentSyscallArg>,
    pub ret: IntentSyscallRet,
    pub safety: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct IntentSyscallArg {
    pub name: String,
    pub r#type: String,
    #[serde(default)]
    pub len: Option<String>,
    #[serde(default)]
    pub check: Option<String>,
    pub doc: String,
}

#[derive(Debug, Deserialize)]
pub struct IntentSyscallRet {
    pub r#type: String,
    pub errno: bool,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IntentAuditCode {
    pub code: String,
    pub name: String,
    pub description: String,
}

pub struct IntentGenerator {
    schema: IntentSchema,
    tera: Tera,
}

impl IntentGenerator {
    pub fn new(schema_path: &Path, template_dir: &Path) -> Result<Self> {
        let content = fs::read_to_string(schema_path)
            .context("Failed to read intent schema")?;
        
        let schema: IntentSchema = serde_yaml::from_str(&content)
            .context("Failed to parse intent schema")?;
        
        let template_pattern = template_dir.join("**/*.tera");
        let tera = Tera::new(template_pattern.to_str().unwrap())
            .context("Failed to load Tera templates")?;
        
        Ok(Self { schema, tera })
    }
    
    pub fn generate_all(&self, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir)?;
        
        self.generate_kernel_wire(output_dir)?;
        self.generate_kernel_sys(output_dir)?;
        self.generate_userland_stubs(output_dir)?;
        self.generate_c_header(output_dir)?;
        self.generate_documentation(output_dir)?;
        
        Ok(())
    }
    
    fn generate_kernel_wire(&self, output_dir: &Path) -> Result<()> {
        let mut context = TeraContext::new();
        context.insert("schema", &self.schema);
        
        let content = self.tera.render("intent_wire.rs.tera", &context)
            .context("Failed to render kernel wire template")?;
        
        let output_path = output_dir.join("kernel/include/intent_wire.rs");
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(&output_path, content)?;
        
        Ok(())
    }
    
    fn generate_kernel_sys(&self, output_dir: &Path) -> Result<()> {
        let mut context = TeraContext::new();
        context.insert("schema", &self.schema);
        
        let content = self.tera.render("intent_sys.rs.tera", &context)
            .context("Failed to render kernel sys template")?;
        
        let output_path = output_dir.join("kernel/src/intent/sys.rs");
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(&output_path, content)?;
        
        Ok(())
    }
    
    fn generate_userland_stubs(&self, output_dir: &Path) -> Result<()> {
        let mut context = TeraContext::new();
        context.insert("schema", &self.schema);
        
        let content = self.tera.render("intent_stubs.rs.tera", &context)
            .context("Failed to render userland stubs template")?;
        
        let output_path = output_dir.join("userland-stubs/src/intent.rs");
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(&output_path, content)?;
        
        Ok(())
    }
    
    fn generate_c_header(&self, output_dir: &Path) -> Result<()> {
        let mut context = TeraContext::new();
        context.insert("schema", &self.schema);
        
        let content = self.tera.render("intent_c.h.tera", &context)
            .context("Failed to render C header template")?;
        
        let output_path = output_dir.join("include/abi/polymera_intent.h");
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(&output_path, content)?;
        
        Ok(())
    }
    
    fn generate_documentation(&self, output_dir: &Path) -> Result<()> {
        let mut context = TeraContext::new();
        context.insert("schema", &self.schema);
        
        let content = self.tera.render("intent_bus.md.tera", &context)
            .context("Failed to render documentation template")?;
        
        let output_path = output_dir.join("docs/abi/INTENT_BUS.md");
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(&output_path, content)?;
        
        Ok(())
    }
    
    pub fn get_syscall_ids(&self) -> Vec<(String, String)> {
        self.schema.syscalls.iter()
            .map(|syscall| (syscall.name.clone(), syscall.id.clone()))
            .collect()
    }
    
    pub fn get_audit_codes(&self) -> Vec<(String, String)> {
        self.schema.audit_codes.iter()
            .map(|code| (code.name.clone(), code.code.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_intent_generator_creation() {
        let temp_dir = tempdir().unwrap();
        let schema_path = temp_dir.path().join("test_intent.yaml");
        
        let test_schema = r#"
version: 1
name: "Test Intent"
description: "Test schema"
lanes: ["RT", "HIGH"]
structs: {}
enums: {}
limits:
  text_max: 1024
  plan_max: 512
  preview_max: 256
  whylog_max: 128
  queue_rt_max: 10
  queue_high_max: 50
  queue_best_max: 100
policy:
  mode_feature: "TEST_POLICY"
  decisions: ["allow", "deny"]
  description: "Test policy"
mapping:
  scopes_to_caps:
    description: "Test mapping"
    mappings: {}
syscalls: []
audit_codes: []
"#;
        
        fs::write(&schema_path, test_schema).unwrap();
        
        let template_dir = tempdir().unwrap();
        let generator = IntentGenerator::new(&schema_path, template_dir.path()).unwrap();
        
        assert_eq!(generator.schema.version, 1);
        assert_eq!(generator.schema.name, "Test Intent");
        assert_eq!(generator.schema.lanes, vec!["RT", "HIGH"]);
    }
}
