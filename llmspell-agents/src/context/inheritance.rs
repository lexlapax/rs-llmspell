//! ABOUTME: Context inheritance rules and field propagation
//! ABOUTME: Defines how context data is inherited between parent and child contexts

use llmspell_core::execution_context::{ExecutionContext, InheritancePolicy};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

/// Field-level inheritance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInheritance {
    /// Fields that should always be inherited
    pub always_inherit: HashSet<String>,
    /// Fields that should never be inherited
    pub never_inherit: HashSet<String>,
    /// Fields that are inherited only with specific policies
    pub conditional_inherit: HashMap<String, Vec<InheritancePolicy>>,
    /// Transform functions for inherited fields
    pub transforms: HashMap<String, FieldTransform>,
}

/// Transformation to apply when inheriting a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldTransform {
    /// Copy as-is
    Copy,
    /// Append a prefix
    Prefix(String),
    /// Append a suffix
    Suffix(String),
    /// Custom transformation (field name)
    Custom(String),
}

impl Default for FieldInheritance {
    fn default() -> Self {
        let mut always_inherit = HashSet::new();
        always_inherit.insert("conversation_id".to_string());
        always_inherit.insert("user_id".to_string());
        always_inherit.insert("session_id".to_string());
        always_inherit.insert("security_level".to_string());

        let mut never_inherit = HashSet::new();
        never_inherit.insert("agent_private_state".to_string());
        never_inherit.insert("temporary_data".to_string());

        Self {
            always_inherit,
            never_inherit,
            conditional_inherit: HashMap::new(),
            transforms: HashMap::new(),
        }
    }
}

/// Rules for context inheritance behavior
pub struct InheritanceRules {
    /// Field-level inheritance configuration
    pub field_rules: FieldInheritance,
    /// Maximum inheritance depth
    pub max_depth: usize,
    /// Whether to merge or replace on conflicts
    pub conflict_resolution: ConflictResolution,
    /// Custom validators
    validators: Vec<Box<dyn InheritanceValidator>>,
}

/// How to resolve conflicts when inheriting data
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Child value takes precedence
    ChildWins,
    /// Parent value takes precedence
    ParentWins,
    /// Merge values (for objects and arrays)
    Merge,
    /// Keep both in an array
    KeepBoth,
}

/// Trait for custom inheritance validation
pub trait InheritanceValidator: Send + Sync {
    /// Validate whether a field should be inherited
    fn should_inherit(&self, field: &str, value: &Value, policy: &InheritancePolicy) -> bool;

    /// Validate the inherited value
    fn validate_value(&self, field: &str, value: &Value) -> bool;
}

impl InheritanceRules {
    /// Create new inheritance rules with defaults
    #[must_use]
    pub fn new() -> Self {
        Self {
            field_rules: FieldInheritance::default(),
            max_depth: 10,
            conflict_resolution: ConflictResolution::ChildWins,
            validators: Vec::new(),
        }
    }

    /// Add a field that should always be inherited
    #[must_use]
    pub fn always_inherit(mut self, field: String) -> Self {
        self.field_rules.always_inherit.insert(field);
        self
    }

    /// Add a field that should never be inherited
    #[must_use]
    pub fn never_inherit(mut self, field: String) -> Self {
        self.field_rules.never_inherit.insert(field);
        self
    }

    /// Add conditional inheritance for a field
    #[must_use]
    pub fn conditional_inherit(mut self, field: String, policies: Vec<InheritancePolicy>) -> Self {
        self.field_rules.conditional_inherit.insert(field, policies);
        self
    }

    /// Add a field transform
    #[must_use]
    pub fn with_transform(mut self, field: String, transform: FieldTransform) -> Self {
        self.field_rules.transforms.insert(field, transform);
        self
    }

    /// Set maximum inheritance depth
    #[must_use]
    pub const fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set conflict resolution strategy
    #[must_use]
    pub const fn conflict_resolution(mut self, resolution: ConflictResolution) -> Self {
        self.conflict_resolution = resolution;
        self
    }

    /// Add a custom validator
    #[must_use]
    pub fn add_validator(mut self, validator: Box<dyn InheritanceValidator>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Apply inheritance rules to create child context data
    pub fn apply(
        &self,
        parent_ctx: &ExecutionContext,
        child_ctx: &mut ExecutionContext,
    ) -> Result<(), String> {
        // Check depth
        if parent_ctx.depth() >= self.max_depth {
            return Err(format!(
                "Maximum inheritance depth {} exceeded",
                self.max_depth
            ));
        }

        // Apply field rules based on inheritance policy
        match child_ctx.inheritance {
            InheritancePolicy::Inherit => {
                self.inherit_all_fields(parent_ctx, child_ctx)?;
            }
            InheritancePolicy::Copy => {
                self.copy_specific_fields(parent_ctx, child_ctx)?;
            }
            InheritancePolicy::Share => {
                // Share mode - data accessed via parent reference
                // No copying needed
            }
            InheritancePolicy::Isolate => {
                // Isolate mode - no inheritance
                // But still inherit always_inherit fields
                self.inherit_required_fields(parent_ctx, child_ctx)?;
            }
        }

        Ok(())
    }

    fn inherit_all_fields(
        &self,
        parent: &ExecutionContext,
        child: &mut ExecutionContext,
    ) -> Result<(), String> {
        for (field, value) in &parent.data {
            if self.should_inherit_field(field, value, &child.inheritance) {
                let transformed_value = self.transform_value(field, value);
                self.set_field_value(child, field.clone(), transformed_value)?;
            }
        }
        Ok(())
    }

    fn copy_specific_fields(
        &self,
        parent: &ExecutionContext,
        child: &mut ExecutionContext,
    ) -> Result<(), String> {
        // Copy always_inherit fields and conditional fields that match the policy
        for field in &self.field_rules.always_inherit {
            if let Some(value) = parent.data.get(field) {
                let transformed_value = self.transform_value(field, value);
                self.set_field_value(child, field.clone(), transformed_value)?;
            }
        }

        for (field, policies) in &self.field_rules.conditional_inherit {
            if policies.contains(&InheritancePolicy::Copy) {
                if let Some(value) = parent.data.get(field) {
                    let transformed_value = self.transform_value(field, value);
                    self.set_field_value(child, field.clone(), transformed_value)?;
                }
            }
        }

        Ok(())
    }

    fn inherit_required_fields(
        &self,
        parent: &ExecutionContext,
        child: &mut ExecutionContext,
    ) -> Result<(), String> {
        for field in &self.field_rules.always_inherit {
            if let Some(value) = parent.data.get(field) {
                let transformed_value = self.transform_value(field, value);
                self.set_field_value(child, field.clone(), transformed_value)?;
            }
        }
        Ok(())
    }

    fn should_inherit_field(&self, field: &str, value: &Value, policy: &InheritancePolicy) -> bool {
        // Check never_inherit first
        if self.field_rules.never_inherit.contains(field) {
            return false;
        }

        // Check always_inherit
        if self.field_rules.always_inherit.contains(field) {
            return true;
        }

        // Check conditional inheritance
        if let Some(policies) = self.field_rules.conditional_inherit.get(field) {
            if !policies.contains(policy) {
                return false;
            }
        }

        // Check custom validators
        for validator in &self.validators {
            if !validator.should_inherit(field, value, policy) {
                return false;
            }
        }

        true
    }

    fn transform_value(&self, field: &str, value: &Value) -> Value {
        match self.field_rules.transforms.get(field) {
            Some(FieldTransform::Prefix(prefix)) => value.as_str().map_or_else(
                || value.clone(),
                |str_val| Value::String(format!("{prefix}{str_val}")),
            ),
            Some(FieldTransform::Suffix(suffix)) => value.as_str().map_or_else(
                || value.clone(),
                |str_val| Value::String(format!("{str_val}{suffix}")),
            ),
            Some(FieldTransform::Copy) | Some(FieldTransform::Custom(_)) | None => value.clone(),
        }
    }

    fn set_field_value(
        &self,
        ctx: &mut ExecutionContext,
        field: String,
        value: Value,
    ) -> Result<(), String> {
        // Validate value
        for validator in &self.validators {
            if !validator.validate_value(&field, &value) {
                return Err(format!("Validation failed for field: {field}"));
            }
        }

        // Handle conflicts
        if let Some(existing) = ctx.data.get(&field) {
            match self.conflict_resolution {
                ConflictResolution::ChildWins => {
                    // Don't overwrite child's value
                    return Ok(());
                }
                ConflictResolution::ParentWins => {
                    ctx.set(field, value);
                }
                ConflictResolution::Merge => {
                    let merged = Self::merge_values(existing, &value);
                    ctx.set(field, merged);
                }
                ConflictResolution::KeepBoth => {
                    let both = json!([existing, value]);
                    ctx.set(field, both);
                }
            }
        } else {
            ctx.set(field, value);
        }

        Ok(())
    }

    fn merge_values(existing: &Value, new: &Value) -> Value {
        match (existing, new) {
            (Value::Object(existing_obj), Value::Object(new_obj)) => {
                let mut merged = existing_obj.clone();
                for (k, v) in new_obj {
                    merged.insert(k.clone(), v.clone());
                }
                Value::Object(merged)
            }
            (Value::Array(existing_arr), Value::Array(new_arr)) => {
                let mut merged = existing_arr.clone();
                merged.extend(new_arr.clone());
                Value::Array(merged)
            }
            _ => new.clone(), // For non-mergeable types, use new value
        }
    }
}

impl Default for InheritanceRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Example validator for security fields
pub struct SecurityFieldValidator;

impl InheritanceValidator for SecurityFieldValidator {
    fn should_inherit(&self, field: &str, _value: &Value, _policy: &InheritancePolicy) -> bool {
        // Security fields should generally be inherited
        field.starts_with("security_") || field == "permissions"
    }

    fn validate_value(&self, field: &str, value: &Value) -> bool {
        if field == "security_level" {
            value.as_str().map_or(false, |level| {
                ["low", "medium", "high", "critical"].contains(&level)
            })
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_default_inheritance_rules() {
        let rules = InheritanceRules::new();

        assert!(rules.field_rules.always_inherit.contains("conversation_id"));
        assert!(rules.field_rules.always_inherit.contains("user_id"));
        assert!(rules
            .field_rules
            .never_inherit
            .contains("agent_private_state"));
    }
    #[test]
    fn test_field_inheritance() {
        let rules = InheritanceRules::new()
            .always_inherit("custom_field".to_string())
            .never_inherit("secret_field".to_string());

        let mut parent = ExecutionContext::new();
        parent.set("custom_field".to_string(), json!("inherited"));
        parent.set("secret_field".to_string(), json!("not inherited"));
        parent.set("normal_field".to_string(), json!("maybe inherited"));

        // Create child with Isolate policy so no data is pre-copied
        let mut child = parent.create_child(
            llmspell_core::execution_context::ContextScope::Session("s1".to_string()),
            InheritancePolicy::Isolate,
        );

        // Now apply rules which should only inherit allowed fields
        rules.apply(&parent, &mut child).unwrap();

        assert_eq!(child.get("custom_field"), Some(json!("inherited")));
        assert_eq!(child.get("secret_field"), None);
        // normal_field should not be inherited with Isolate policy
        assert_eq!(child.get("normal_field"), None);
    }
    #[test]
    fn test_field_transforms() {
        let rules = InheritanceRules::new()
            .always_inherit("prefixed_field".to_string())
            .with_transform(
                "prefixed_field".to_string(),
                FieldTransform::Prefix("child_".to_string()),
            );

        let mut parent = ExecutionContext::new();
        parent.set("prefixed_field".to_string(), json!("value"));

        // Use Isolate policy to ensure clean slate
        let mut child = parent.create_child(
            llmspell_core::execution_context::ContextScope::Session("s1".to_string()),
            InheritancePolicy::Isolate,
        );

        // Apply rules to inherit and transform the field
        rules.apply(&parent, &mut child).unwrap();

        assert_eq!(child.get("prefixed_field"), Some(json!("child_value")));
    }
    #[test]
    fn test_conflict_resolution() {
        // Test ChildWins
        let rules = InheritanceRules::new().conflict_resolution(ConflictResolution::ChildWins);

        let mut parent = ExecutionContext::new();
        parent.set("field".to_string(), json!("parent_value"));

        let mut child = parent.create_child(
            llmspell_core::execution_context::ContextScope::Session("s1".to_string()),
            InheritancePolicy::Inherit,
        );
        child.set("field".to_string(), json!("child_value"));

        rules.apply(&parent, &mut child).unwrap();

        assert_eq!(child.get("field"), Some(json!("child_value")));

        // Test Merge
        let rules = InheritanceRules::new().conflict_resolution(ConflictResolution::Merge);

        let mut parent = ExecutionContext::new();
        parent.set("data".to_string(), json!({"a": 1}));

        let mut child = parent.create_child(
            llmspell_core::execution_context::ContextScope::Session("s2".to_string()),
            InheritancePolicy::Inherit,
        );
        child.set("data".to_string(), json!({"b": 2}));

        rules.apply(&parent, &mut child).unwrap();

        let merged = child.get("data").unwrap();
        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], 2);
    }
    #[test]
    fn test_security_validator() {
        let mut rules = InheritanceRules::new();
        rules = rules.add_validator(Box::new(SecurityFieldValidator));

        let mut parent = ExecutionContext::new();
        parent.set("security_level".to_string(), json!("high"));
        parent.set("security_invalid".to_string(), json!("invalid_level"));

        let mut child = parent.create_child(
            llmspell_core::execution_context::ContextScope::Session("s1".to_string()),
            InheritancePolicy::Inherit,
        );

        let result = rules.apply(&parent, &mut child);

        // Should succeed for valid security level
        assert!(result.is_ok());
        assert_eq!(child.get("security_level"), Some(json!("high")));
    }
}
