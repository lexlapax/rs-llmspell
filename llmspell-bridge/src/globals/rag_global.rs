//! ABOUTME: RAG global object implementation for script engines
//! ABOUTME: Provides vector storage and retrieval functionality

use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::rag_bridge::RAGBridge;
use crate::{ComponentRegistry, ProviderManager};
use llmspell_core::traits::storage::VectorStorage;
use llmspell_core::Result;
use llmspell_kernel::sessions::SessionManager;
use llmspell_kernel::state::StateManager;
use llmspell_rag::multi_tenant_integration::MultiTenantRAG;
use std::sync::Arc;

/// RAG global object for script engines
pub struct RAGGlobal {
    bridge: Arc<RAGBridge>,
    #[allow(dead_code)]
    registry: Arc<ComponentRegistry>,
    #[allow(dead_code)]
    providers: Arc<ProviderManager>,
}

impl RAGGlobal {
    /// Create a new RAG global
    ///
    /// # Errors
    ///
    /// Returns an error if dependencies are not available
    pub async fn new(
        registry: Arc<ComponentRegistry>,
        providers: Arc<ProviderManager>,
        state_manager: Arc<StateManager>,
        session_manager: Arc<SessionManager>,
        multi_tenant_rag: Arc<MultiTenantRAG>,
        vector_storage: Option<Arc<dyn VectorStorage>>,
    ) -> Result<Self> {
        // Create provider manager for RAG operations
        let core_providers = providers.create_core_manager_arc().await?;

        let bridge = Arc::new(RAGBridge::new(
            state_manager,
            session_manager,
            multi_tenant_rag,
            core_providers,
            vector_storage,
        ));

        Ok(Self {
            bridge,
            registry,
            providers,
        })
    }

    /// Create with minimal dependencies (for testing)
    ///
    /// # Errors
    ///
    /// Returns an error if dependencies are not available
    pub async fn with_managers(
        registry: Arc<ComponentRegistry>,
        providers: Arc<ProviderManager>,
        state_manager: Arc<StateManager>,
        session_manager: Arc<SessionManager>,
        multi_tenant_rag: Arc<MultiTenantRAG>,
        vector_storage: Option<Arc<dyn VectorStorage>>,
    ) -> Result<Self> {
        let core_providers = providers.create_core_manager_arc().await?;

        let bridge = Arc::new(RAGBridge::new(
            state_manager,
            session_manager,
            multi_tenant_rag,
            core_providers,
            vector_storage,
        ));

        Ok(Self {
            bridge,
            registry,
            providers,
        })
    }

    /// Get the RAG bridge
    #[must_use]
    pub const fn bridge(&self) -> &Arc<RAGBridge> {
        &self.bridge
    }
}

impl GlobalObject for RAGGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "RAG".to_string(),
            description: "Retrieval-Augmented Generation with vector storage".to_string(),
            dependencies: vec!["State".to_string(), "Session".to_string()],
            required: false,
            version: "1.0.0".to_string(),
        }
    }

    fn initialize(&self, context: &GlobalContext) -> Result<()> {
        // Store bridge reference for cross-global access
        context.set_bridge("rag", self.bridge.clone());
        Ok(())
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::rag::inject_rag_global(lua, context, self.bridge.clone()).map_err(
            |e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject RAG global: {e}"),
                source: None,
            },
        )
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        ctx: &mut boa_engine::Context,
        context: &GlobalContext,
    ) -> Result<()> {
        crate::javascript::globals::rag::inject_rag_global(ctx, context).map_err(|e| {
            llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject RAG global for JavaScript: {e}"),
                source: None,
            }
        })
    }
}
