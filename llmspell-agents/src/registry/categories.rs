//! ABOUTME: Agent categorization and tagging system
//! ABOUTME: Provides hierarchical categories and flexible tagging

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Agent category definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentCategory {
    /// Category ID
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Parent category ID (for hierarchy)
    pub parent_id: Option<String>,

    /// Category metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Predefined agent categories
pub struct StandardCategories;

impl StandardCategories {
    /// Tool-using agents
    #[must_use]
    pub fn tool_agents() -> AgentCategory {
        AgentCategory {
            id: "tool-agents".to_string(),
            name: "Tool Agents".to_string(),
            description: "Agents that can use tools to perform tasks".to_string(),
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// LLM-based agents
    #[must_use]
    pub fn llm_agents() -> AgentCategory {
        AgentCategory {
            id: "llm-agents".to_string(),
            name: "LLM Agents".to_string(),
            description: "Agents powered by language models".to_string(),
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Workflow orchestration agents
    #[must_use]
    pub fn workflow_agents() -> AgentCategory {
        AgentCategory {
            id: "workflow-agents".to_string(),
            name: "Workflow Agents".to_string(),
            description: "Agents that orchestrate workflows and processes".to_string(),
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Monitoring agents
    #[must_use]
    pub fn monitoring_agents() -> AgentCategory {
        AgentCategory {
            id: "monitoring-agents".to_string(),
            name: "Monitoring Agents".to_string(),
            description: "Agents that monitor systems and processes".to_string(),
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Security agents
    #[must_use]
    pub fn security_agents() -> AgentCategory {
        AgentCategory {
            id: "security-agents".to_string(),
            name: "Security Agents".to_string(),
            description: "Agents focused on security tasks".to_string(),
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Data processing agents
    #[must_use]
    pub fn data_agents() -> AgentCategory {
        AgentCategory {
            id: "data-agents".to_string(),
            name: "Data Processing Agents".to_string(),
            description: "Agents that process and transform data".to_string(),
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Research agents
    #[must_use]
    pub fn research_agents() -> AgentCategory {
        AgentCategory {
            id: "research-agents".to_string(),
            name: "Research Agents".to_string(),
            description: "Agents that gather and analyze information".to_string(),
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Communication agents
    #[must_use]
    pub fn communication_agents() -> AgentCategory {
        AgentCategory {
            id: "communication-agents".to_string(),
            name: "Communication Agents".to_string(),
            description: "Agents that handle communication tasks".to_string(),
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Get all standard categories
    #[must_use]
    pub fn all() -> Vec<AgentCategory> {
        vec![
            Self::tool_agents(),
            Self::llm_agents(),
            Self::workflow_agents(),
            Self::monitoring_agents(),
            Self::security_agents(),
            Self::data_agents(),
            Self::research_agents(),
            Self::communication_agents(),
        ]
    }
}

/// Tag for flexible agent labeling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentTag {
    /// Tag name
    pub name: String,

    /// Tag value (optional)
    pub value: Option<String>,

    /// Tag type
    pub tag_type: TagType,
}

/// Type of agent tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TagType {
    /// Feature tag
    Feature,

    /// Capability tag
    Capability,

    /// Environment tag
    Environment,

    /// Version tag
    Version,

    /// Custom tag
    Custom,
}

/// Category manager for organizing agents
pub struct CategoryManager {
    categories: HashMap<String, AgentCategory>,
    agent_categories: HashMap<String, HashSet<String>>,
    agent_tags: HashMap<String, Vec<AgentTag>>,
}

impl CategoryManager {
    /// Create new category manager
    #[must_use]
    pub fn new() -> Self {
        let mut manager = Self {
            categories: HashMap::new(),
            agent_categories: HashMap::new(),
            agent_tags: HashMap::new(),
        };

        // Add standard categories
        for category in StandardCategories::all() {
            manager.add_category(category).ok();
        }

        manager
    }

    /// Add a category
    ///
    /// # Errors
    ///
    /// Returns an error if the category already exists
    pub fn add_category(&mut self, category: AgentCategory) -> Result<()> {
        if self.categories.contains_key(&category.id) {
            anyhow::bail!("Category '{}' already exists", category.id);
        }

        // Validate parent exists if specified
        if let Some(parent_id) = &category.parent_id {
            if !self.categories.contains_key(parent_id) {
                anyhow::bail!("Parent category '{}' not found", parent_id);
            }
        }

        self.categories.insert(category.id.clone(), category);
        Ok(())
    }

    /// Get category by ID
    #[must_use]
    pub fn get_category(&self, id: &str) -> Option<&AgentCategory> {
        self.categories.get(id)
    }

    /// List all categories
    #[must_use]
    pub fn list_categories(&self) -> Vec<&AgentCategory> {
        self.categories.values().collect()
    }

    /// Get category hierarchy
    #[must_use]
    pub fn get_hierarchy(&self, category_id: &str) -> Vec<String> {
        let mut hierarchy = Vec::new();
        let mut current_id = Some(category_id.to_string());

        while let Some(id) = current_id {
            hierarchy.push(id.clone());
            current_id = self.categories.get(&id).and_then(|c| c.parent_id.clone());
        }

        hierarchy.reverse();
        hierarchy
    }

    /// Assign agent to category
    ///
    /// # Errors
    ///
    /// Returns an error if the category is not found
    pub fn assign_to_category(&mut self, agent_id: &str, category_id: &str) -> Result<()> {
        if !self.categories.contains_key(category_id) {
            anyhow::bail!("Category '{}' not found", category_id);
        }

        self.agent_categories
            .entry(agent_id.to_string())
            .or_default()
            .insert(category_id.to_string());

        Ok(())
    }

    /// Remove agent from category
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn remove_from_category(&mut self, agent_id: &str, category_id: &str) -> Result<()> {
        if let Some(categories) = self.agent_categories.get_mut(agent_id) {
            categories.remove(category_id);
        }
        Ok(())
    }

    /// Get agent categories
    #[must_use]
    pub fn get_agent_categories(&self, agent_id: &str) -> Vec<&str> {
        self.agent_categories
            .get(agent_id)
            .map(|cats| cats.iter().map(std::string::String::as_str).collect())
            .unwrap_or_default()
    }

    /// Find agents in category
    #[must_use]
    pub fn find_agents_in_category(&self, category_id: &str) -> Vec<&str> {
        self.agent_categories
            .iter()
            .filter_map(|(agent_id, categories)| {
                if categories.contains(category_id) {
                    Some(agent_id.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Add tag to agent
    ///
    /// # Errors
    ///
    /// Returns an error if tag addition fails
    pub fn add_tag(&mut self, agent_id: &str, tag: AgentTag) -> Result<()> {
        let tags = self.agent_tags.entry(agent_id.to_string()).or_default();

        // Only add if not already present
        if !tags.iter().any(|t| t == &tag) {
            tags.push(tag);
        }

        Ok(())
    }

    /// Remove tag from agent
    ///
    /// # Errors
    ///
    /// Returns an error if tag removal fails
    pub fn remove_tag(&mut self, agent_id: &str, tag: &AgentTag) -> Result<()> {
        if let Some(tags) = self.agent_tags.get_mut(agent_id) {
            tags.retain(|t| t != tag);
        }
        Ok(())
    }

    /// Get agent tags
    #[must_use]
    pub fn get_agent_tags(&self, agent_id: &str) -> Vec<&AgentTag> {
        self.agent_tags
            .get(agent_id)
            .map(|tags| tags.iter().collect())
            .unwrap_or_default()
    }

    /// Find agents by tag
    #[must_use]
    pub fn find_agents_by_tag(&self, tag_name: &str) -> Vec<&str> {
        self.agent_tags
            .iter()
            .filter_map(|(agent_id, tags)| {
                if tags.iter().any(|t| t.name == tag_name) {
                    Some(agent_id.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for CategoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Category builder for hierarchical categories
pub struct CategoryBuilder {
    id: String,
    name: String,
    description: String,
    parent_id: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl CategoryBuilder {
    /// Create new category builder
    #[must_use]
    pub fn new(id: String) -> Self {
        Self {
            id: id.clone(),
            name: id,
            description: String::new(),
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Set name
    #[must_use]
    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// Set description
    #[must_use]
    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Set parent category
    #[must_use]
    pub fn parent(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Add metadata
    #[must_use]
    pub fn metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Build category
    #[must_use]
    pub fn build(self) -> AgentCategory {
        AgentCategory {
            id: self.id,
            name: self.name,
            description: self.description,
            parent_id: self.parent_id,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_hierarchy() {
        let mut manager = CategoryManager::new();

        // Add subcategory
        let subcategory = CategoryBuilder::new("llm-chatbots".to_string())
            .name("LLM Chatbots".to_string())
            .description("Chatbot agents powered by LLMs".to_string())
            .parent("llm-agents".to_string())
            .build();

        manager.add_category(subcategory).unwrap();

        let hierarchy = manager.get_hierarchy("llm-chatbots");
        assert_eq!(hierarchy, vec!["llm-agents", "llm-chatbots"]);
    }

    #[test]
    fn test_agent_categorization() {
        let mut manager = CategoryManager::new();

        manager
            .assign_to_category("agent-1", "tool-agents")
            .unwrap();
        manager
            .assign_to_category("agent-1", "research-agents")
            .unwrap();

        let categories = manager.get_agent_categories("agent-1");
        assert_eq!(categories.len(), 2);

        let agents = manager.find_agents_in_category("tool-agents");
        assert_eq!(agents, vec!["agent-1"]);
    }

    #[test]
    fn test_agent_tagging() {
        let mut manager = CategoryManager::new();

        let tag1 = AgentTag {
            name: "version".to_string(),
            value: Some("1.0.0".to_string()),
            tag_type: TagType::Version,
        };

        let tag2 = AgentTag {
            name: "production".to_string(),
            value: None,
            tag_type: TagType::Environment,
        };

        manager.add_tag("agent-1", tag1).unwrap();
        manager.add_tag("agent-1", tag2).unwrap();

        let agent_tags = manager.get_agent_tags("agent-1");
        assert_eq!(agent_tags.len(), 2);

        let agents = manager.find_agents_by_tag("production");
        assert_eq!(agents, vec!["agent-1"]);
    }
}
