//! ABOUTME: Global registry for managing all injectable global objects
//! ABOUTME: Handles registration, dependency resolution, and lifecycle management

use super::types::{GlobalMetadata, GlobalObject, InjectionMetrics};
use llmspell_core::{LLMSpellError, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Registry for all global objects
pub struct GlobalRegistry {
    /// Registered globals by name
    globals: HashMap<String, Arc<dyn GlobalObject>>,
    /// Injection order based on dependency resolution
    injection_order: Vec<String>,
    /// Performance metrics
    metrics: InjectionMetrics,
}

impl std::fmt::Debug for GlobalRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalRegistry")
            .field("global_count", &self.globals.len())
            .field("injection_order", &self.injection_order)
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl GlobalRegistry {
    /// Get a global by name
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Arc<dyn GlobalObject>> {
        self.globals.get(name).cloned()
    }

    /// Get all registered globals in injection order
    #[must_use]
    pub fn get_all_ordered(&self) -> Vec<Arc<dyn GlobalObject>> {
        self.injection_order
            .iter()
            .filter_map(|name| self.globals.get(name).cloned())
            .collect()
    }

    /// Get injection metrics
    #[must_use]
    pub const fn metrics(&self) -> &InjectionMetrics {
        &self.metrics
    }

    /// Get metadata for all registered globals
    #[must_use]
    pub fn list_globals(&self) -> Vec<GlobalMetadata> {
        self.injection_order
            .iter()
            .filter_map(|name| self.globals.get(name).map(|g| g.metadata()))
            .collect()
    }
}

/// Builder for creating a `GlobalRegistry` with dependency resolution
pub struct GlobalRegistryBuilder {
    globals: HashMap<String, Arc<dyn GlobalObject>>,
}

impl GlobalRegistryBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }

    /// Register a global object
    pub fn register(&mut self, global: Arc<dyn GlobalObject>) -> &mut Self {
        let metadata = global.metadata();
        self.globals.insert(metadata.name, global);
        self
    }

    /// Build the registry with dependency resolution
    pub fn build(self) -> Result<GlobalRegistry> {
        let injection_order = self.resolve_dependencies()?;

        Ok(GlobalRegistry {
            globals: self.globals,
            injection_order,
            metrics: InjectionMetrics::default(),
        })
    }

    /// Resolve dependencies and return injection order
    fn resolve_dependencies(&self) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for name in self.globals.keys() {
            if !visited.contains(name) {
                self.visit(name, &mut visited, &mut visiting, &mut order)?;
            }
        }

        Ok(order)
    }

    /// DFS visit for topological sort
    fn visit(
        &self,
        name: &str,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if visiting.contains(name) {
            return Err(LLMSpellError::Component {
                message: format!("Circular dependency detected involving global: {name}"),
                source: None,
            });
        }

        if visited.contains(name) {
            return Ok(());
        }

        visiting.insert(name.to_string());

        if let Some(global) = self.globals.get(name) {
            let metadata = global.metadata();
            for dep in &metadata.dependencies {
                if !self.globals.contains_key(dep) {
                    // Skip missing optional dependencies
                    if let Some(dep_global) = self.globals.get(dep) {
                        if !dep_global.metadata().required {
                            continue;
                        }
                    }
                    return Err(LLMSpellError::Component {
                        message: format!(
                            "Global '{name}' depends on '{dep}' which is not registered"
                        ),
                        source: None,
                    });
                }
                self.visit(dep, visited, visiting, order)?;
            }
        }

        visiting.remove(name);
        visited.insert(name.to_string());
        order.push(name.to_string());

        Ok(())
    }
}

impl Default for GlobalRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::globals::types::GlobalContext;

    // Mock global for testing
    struct MockGlobal {
        name: String,
        deps: Vec<String>,
    }

    impl GlobalObject for MockGlobal {
        fn metadata(&self) -> GlobalMetadata {
            GlobalMetadata {
                name: self.name.clone(),
                description: "Mock global".to_string(),
                dependencies: self.deps.clone(),
                required: true,
                version: "1.0.0".to_string(),
            }
        }

        #[cfg(feature = "lua")]
        fn inject_lua(&self, _lua: &mlua::Lua, _context: &GlobalContext) -> Result<()> {
            Ok(())
        }

        #[cfg(feature = "javascript")]
        fn inject_javascript(
            &self,
            _ctx: &mut boa_engine::Context,
            _context: &GlobalContext,
        ) -> Result<()> {
            Ok(())
        }
    }
    #[test]
    fn test_dependency_resolution() {
        let mut builder = GlobalRegistryBuilder::new();

        // A depends on B, B depends on C
        builder.register(Arc::new(MockGlobal {
            name: "A".to_string(),
            deps: vec!["B".to_string()],
        }));
        builder.register(Arc::new(MockGlobal {
            name: "B".to_string(),
            deps: vec!["C".to_string()],
        }));
        builder.register(Arc::new(MockGlobal {
            name: "C".to_string(),
            deps: vec![],
        }));

        let registry = builder.build().unwrap();

        // Should be injected in order: C, B, A
        assert_eq!(registry.injection_order, vec!["C", "B", "A"]);
    }
    #[test]
    fn test_circular_dependency_detection() {
        let mut builder = GlobalRegistryBuilder::new();

        // A depends on B, B depends on A (circular)
        builder.register(Arc::new(MockGlobal {
            name: "A".to_string(),
            deps: vec!["B".to_string()],
        }));
        builder.register(Arc::new(MockGlobal {
            name: "B".to_string(),
            deps: vec!["A".to_string()],
        }));

        let result = builder.build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }
}
