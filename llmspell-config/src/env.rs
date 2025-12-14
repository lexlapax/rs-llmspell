//! ABOUTME: Centralized environment variable registry for LLMSpell configuration
//! ABOUTME: Single source of truth for all environment variable handling

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Category of environment variable
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvCategory {
    /// Core runtime configuration
    Runtime,
    /// Provider-specific configuration
    Provider,
    /// Tool configuration
    Tool,
    /// State persistence configuration
    State,
    /// Session management configuration
    Session,
    /// Hook system configuration
    Hook,
    /// Path discovery configuration
    Path,
}

/// Isolation mode for environment variable handling
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationMode {
    /// Use process environment (default)
    #[default]
    Global,
    /// Ignore process env, use overrides only
    Isolated,
    /// Overrides on top of process env
    Layered,
    /// Tenant-specific isolation
    Tenant(String),
}

/// Validator function type alias to reduce complexity
type ValidatorFn = Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// Definition of an environment variable
pub struct EnvVarDef {
    /// Name of the environment variable
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Category for grouping
    pub category: EnvCategory,
    /// Default value if not set
    pub default: Option<String>,
    /// Validator function
    pub validator: ValidatorFn,
    /// Config path for automatic application (e.g., "runtime.max_concurrent_scripts")
    pub config_path: Option<String>,
    /// Whether this contains sensitive data (for masking)
    pub sensitive: bool,
}

/// Centralized environment variable registry
pub struct EnvRegistry {
    /// All registered environment variable definitions
    definitions: Arc<RwLock<HashMap<String, EnvVarDef>>>,
    /// Programmatic overrides
    overrides: Arc<RwLock<HashMap<String, String>>>,
    /// Current isolation mode
    isolation_mode: IsolationMode,
    /// Cached values from environment
    cached_values: Arc<RwLock<HashMap<String, String>>>,
}

/// (Name, Description, Category, IsSensitive, ConfigPath)
pub type EnvVarInfo = (String, String, EnvCategory, bool, Option<String>);

impl EnvRegistry {
    /// Create a new environment registry
    pub fn new() -> Self {
        Self::with_isolation(IsolationMode::Global)
    }

    /// Create a registry with specific isolation mode
    pub fn with_isolation(mode: IsolationMode) -> Self {
        Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
            overrides: Arc::new(RwLock::new(HashMap::new())),
            isolation_mode: mode,
            cached_values: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new environment variable definition
    pub fn register_var(&self, def: EnvVarDef) -> Result<(), String> {
        let mut defs = self.definitions.write().map_err(|e| e.to_string())?;
        if defs.contains_key(&def.name) {
            return Err(format!(
                "Environment variable {} already registered",
                def.name
            ));
        }
        defs.insert(def.name.clone(), def);
        Ok(())
    }

    /// Load all registered variables from the environment
    pub fn load_from_env(&self) -> Result<(), String> {
        match self.isolation_mode {
            IsolationMode::Isolated | IsolationMode::Tenant(_) => {
                // Don't load from environment in isolated modes
                return Ok(());
            }
            _ => {}
        }

        let defs = self.definitions.read().map_err(|e| e.to_string())?;
        let mut cached = self.cached_values.write().map_err(|e| e.to_string())?;

        for (name, def) in defs.iter() {
            match std::env::var(name) {
                Ok(value) => {
                    // Validate the value
                    (def.validator)(&value)?;
                    cached.insert(name.clone(), value);
                }
                Err(_) => {
                    // Do NOT insert defaults into cached values
                    // Defaults should only be used when no TOML value exists
                    // and no environment variable is set
                }
            }
        }

        Ok(())
    }

    /// Get all effective values (with priority: overrides > cached > default)
    pub fn get_all_values(&self) -> Result<HashMap<String, String>, String> {
        let defs = self.definitions.read().map_err(|e| e.to_string())?;
        let cached = self.cached_values.read().map_err(|e| e.to_string())?;
        let overrides = self.overrides.read().map_err(|e| e.to_string())?;

        let mut values = HashMap::new();

        for (name, def) in defs.iter() {
            // Priority: overrides > cached > default
            if let Some(value) = overrides
                .get(name)
                .or_else(|| cached.get(name))
                .or(def.default.as_ref())
            {
                values.insert(name.clone(), value.clone());
            }
        }

        Ok(values)
    }

    /// Build configuration from environment values (only actually-set values, not defaults)
    pub fn build_config(&self) -> Result<Value, String> {
        // IMPORTANT: Only get values that were actually set, not defaults
        let cached = self.cached_values.read().map_err(|e| e.to_string())?;
        let overrides = self.overrides.read().map_err(|e| e.to_string())?;
        let defs = self.definitions.read().map_err(|e| e.to_string())?;

        let mut config = serde_json::json!({});

        // Process overrides first (highest priority)
        for (name, value) in overrides.iter() {
            if let Some(def) = defs.get(name) {
                // Validate the value
                (def.validator)(value)?;

                // Apply to config JSON structure if path is defined
                if let Some(path) = &def.config_path {
                    apply_to_json_path(&mut config, path, value)?;
                }
            }
        }

        // Then process cached values (actual env vars that were set)
        for (name, value) in cached.iter() {
            // Skip if already set by override
            if overrides.contains_key(name) {
                continue;
            }

            if let Some(def) = defs.get(name) {
                // Validate the value
                (def.validator)(value)?;

                // Apply to config JSON structure if path is defined
                if let Some(path) = &def.config_path {
                    apply_to_json_path(&mut config, path, value)?;
                }
            }
        }

        // NOTE: We intentionally DO NOT use defaults here
        // Defaults should come from the config structs, not env vars

        Ok(config)
    }

    /// Check if a variable is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.definitions
            .read()
            .map(|defs| defs.contains_key(name))
            .unwrap_or(false)
    }

    /// List all registered variables
    pub fn list_vars(&self) -> Result<Vec<EnvVarInfo>, String> {
        let defs = self.definitions.read().map_err(|e| e.to_string())?;
        let mut vars: Vec<_> = defs
            .values()
            .map(|def| {
                (
                    def.name.clone(),
                    def.description.clone(),
                    def.category.clone(),
                    def.sensitive,
                    def.config_path.clone(),
                )
            })
            .collect();
        vars.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(vars)
    }

    /// Validate all loaded values
    pub fn validate_all(&self) -> Result<(), String> {
        let defs = self.definitions.read().map_err(|e| e.to_string())?;
        let cached = self.cached_values.read().map_err(|e| e.to_string())?;
        let overrides = self.overrides.read().map_err(|e| e.to_string())?;

        for (name, def) in defs.iter() {
            if let Some(value) = overrides.get(name).or_else(|| cached.get(name)) {
                (def.validator)(value)?;
            }
        }

        Ok(())
    }

    /// Set programmatic overrides
    pub fn with_overrides(&self, overrides: HashMap<String, String>) -> Result<(), String> {
        let mut current = self.overrides.write().map_err(|e| e.to_string())?;
        current.extend(overrides);
        Ok(())
    }

    /// Create an isolated registry for library mode
    pub fn isolated() -> Self {
        Self::with_isolation(IsolationMode::Isolated)
    }

    /// Get the value of an environment variable
    pub fn get(&self, name: &str) -> Option<String> {
        let overrides = self.overrides.read().ok()?;
        let cached = self.cached_values.read().ok()?;
        let defs = self.definitions.read().ok()?;

        overrides
            .get(name)
            .or_else(|| cached.get(name))
            .or_else(|| defs.get(name).and_then(|d| d.default.as_ref()))
            .cloned()
    }

    /// Clear all cached values (for reload)
    pub fn clear_cache(&self) -> Result<(), String> {
        let mut cached = self.cached_values.write().map_err(|e| e.to_string())?;
        cached.clear();
        Ok(())
    }

    /// Reload environment variables
    pub fn reload(&self) -> Result<(), String> {
        self.clear_cache()?;
        self.load_from_env()?;
        Ok(())
    }
}

impl Default for EnvRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to apply a value to a JSON path
fn apply_to_json_path(config: &mut Value, path: &str, value: &str) -> Result<(), String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = config;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let Some(obj) = current.as_object_mut() {
                // Try to parse as appropriate type
                let parsed_value = if let Ok(b) = value.parse::<bool>() {
                    Value::Bool(b)
                } else if let Ok(n) = value.parse::<i64>() {
                    Value::Number(serde_json::Number::from(n))
                } else if let Ok(f) = value.parse::<f64>() {
                    serde_json::Number::from_f64(f)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::String(value.to_string()))
                } else {
                    Value::String(value.to_string())
                };
                obj.insert(part.to_string(), parsed_value);
            } else {
                return Err(format!("Cannot set {} on non-object", part));
            }
        } else {
            // Intermediate part - ensure object exists
            if !current.as_object().is_some_and(|o| o.contains_key(*part)) {
                if let Some(obj) = current.as_object_mut() {
                    obj.insert(part.to_string(), Value::Object(serde_json::Map::new()));
                }
            }
            current = current
                .get_mut(*part)
                .ok_or_else(|| format!("Failed to navigate to {}", part))?;
        }
    }

    Ok(())
}

/// Builder for EnvVarDef
pub struct EnvVarDefBuilder {
    name: String,
    description: String,
    category: EnvCategory,
    default: Option<String>,
    validator: Option<ValidatorFn>,
    config_path: Option<String>,
    sensitive: bool,
}

impl EnvVarDefBuilder {
    /// Create a new builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            category: EnvCategory::Runtime,
            default: None,
            validator: None,
            config_path: None,
            sensitive: false,
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set category
    pub fn category(mut self, cat: EnvCategory) -> Self {
        self.category = cat;
        self
    }

    /// Set default value
    pub fn default(mut self, val: impl Into<String>) -> Self {
        self.default = Some(val.into());
        self
    }

    /// Set validator function
    pub fn validator<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.validator = Some(Box::new(f));
        self
    }

    /// Set config path for automatic application
    pub fn config_path(mut self, path: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Mark as sensitive
    pub fn sensitive(mut self) -> Self {
        self.sensitive = true;
        self
    }

    /// Build the definition
    pub fn build(self) -> EnvVarDef {
        EnvVarDef {
            name: self.name,
            description: self.description,
            category: self.category,
            default: self.default,
            validator: self.validator.unwrap_or_else(|| Box::new(|_| Ok(()))),
            config_path: self.config_path,
            sensitive: self.sensitive,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_registry_basic() {
        let registry = EnvRegistry::isolated();

        let def = EnvVarDefBuilder::new("TEST_VAR")
            .description("Test variable")
            .category(EnvCategory::Runtime)
            .default("default_value")
            .validator(|v| {
                if v.is_empty() {
                    Err("Value cannot be empty".to_string())
                } else {
                    Ok(())
                }
            })
            .build();

        registry.register_var(def).unwrap();

        let vars = registry.list_vars().unwrap();
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].0, "TEST_VAR");

        let value = registry.get("TEST_VAR");
        assert_eq!(value, Some("default_value".to_string()));
    }

    #[test]
    fn test_env_registry_overrides() {
        let registry = EnvRegistry::isolated();

        let def = EnvVarDefBuilder::new("TEST_VAR")
            .description("Test variable")
            .default("default")
            .build();

        registry.register_var(def).unwrap();

        let mut overrides = HashMap::new();
        overrides.insert("TEST_VAR".to_string(), "overridden".to_string());
        registry.with_overrides(overrides).unwrap();

        let value = registry.get("TEST_VAR");
        assert_eq!(value, Some("overridden".to_string()));
    }
}
