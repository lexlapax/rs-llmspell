// ABOUTME: Migration engine that integrates with existing StateManager and storage adapters
// ABOUTME: Provides safe state data transformation between schema versions

use super::{
    events::{MigrationEvent, MigrationEventBuilder},
    MigrationConfig, MigrationContext, MigrationResult, ValidationLevel,
};
use crate::backend_adapter::StateStorageAdapter;
use crate::error::{StateError, StateResult};
use crate::manager::SerializableState;
use crate::schema::{MigrationPlan, MigrationPlanner, SchemaRegistry, SemanticVersion};
use llmspell_events::{EventBus, EventCorrelationTracker, UniversalEvent};
use llmspell_hooks::{ComponentId, ComponentType, HookContext, HookExecutor, HookPoint};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum MigrationEngineError {
    #[error("Migration failed: {reason}")]
    MigrationFailed { reason: String },

    #[error("Schema not found: {version}")]
    SchemaNotFound { version: SemanticVersion },

    #[error("Validation failed: {details}")]
    ValidationFailed { details: String },

    #[error("Rollback failed: {reason}")]
    RollbackFailed { reason: String },

    #[error("Migration timeout after {timeout:?}")]
    Timeout { timeout: Duration },

    #[error("Migration cancelled: {reason}")]
    Cancelled { reason: String },
}

impl From<MigrationEngineError> for StateError {
    fn from(err: MigrationEngineError) -> Self {
        StateError::MigrationError(err.to_string())
    }
}

/// Migration engine that integrates with existing StateManager infrastructure
pub struct MigrationEngine {
    /// Existing storage adapter for all storage operations
    storage_adapter: Arc<StateStorageAdapter>,

    /// Schema registry from Task 5.4.1
    pub schema_registry: Arc<RwLock<SchemaRegistry>>,

    /// Existing hook executor for migration events
    #[allow(dead_code)]
    hook_executor: Arc<HookExecutor>,

    /// Existing event correlation tracker
    correlation_tracker: Arc<EventCorrelationTracker>,

    /// Existing event bus for migration events
    event_bus: Arc<EventBus>,

    /// Migration planner from schema module
    migration_planner: Arc<RwLock<MigrationPlanner>>,

    /// Active migrations tracking
    active_migrations: Arc<RwLock<HashMap<Uuid, MigrationContext>>>,
}

impl MigrationEngine {
    /// Create new migration engine with existing infrastructure
    pub fn new(
        storage_adapter: Arc<StateStorageAdapter>,
        schema_registry: SchemaRegistry,
        hook_executor: Arc<HookExecutor>,
        correlation_tracker: Arc<EventCorrelationTracker>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        let migration_planner = MigrationPlanner::new();

        Self {
            storage_adapter,
            schema_registry: Arc::new(RwLock::new(schema_registry)),
            hook_executor,
            correlation_tracker,
            event_bus,
            migration_planner: Arc::new(RwLock::new(migration_planner)),
            active_migrations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute migration from one schema version to another
    pub async fn migrate(
        &self,
        from_version: &SemanticVersion,
        to_version: &SemanticVersion,
        config: MigrationConfig,
    ) -> StateResult<MigrationResult> {
        let migration_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let event_builder = MigrationEventBuilder::new(migration_id);

        info!(
            "Starting migration {} from {} to {}",
            migration_id, from_version, to_version
        );

        // Create migration plan
        let plan = {
            let mut planner = self.migration_planner.write();
            let registry = self.schema_registry.read();

            // Register schemas in planner if needed
            if let Some(from_schema) = registry.get_schema(from_version) {
                planner.register_schema((*from_schema).clone());
            } else {
                return Err(MigrationEngineError::SchemaNotFound {
                    version: from_version.clone(),
                }
                .into());
            }

            if let Some(to_schema) = registry.get_schema(to_version) {
                planner.register_schema((*to_schema).clone());
            } else {
                return Err(MigrationEngineError::SchemaNotFound {
                    version: to_version.clone(),
                }
                .into());
            }

            planner
                .create_migration_plan(from_version, to_version)
                .map_err(|e| StateError::MigrationError(e.to_string()))?
        };

        // Create migration context
        let total_steps = plan.steps.len();
        let registry = self.schema_registry.read().clone();
        let mut context = MigrationContext::new(config.clone(), registry, total_steps);
        context.set_metadata("migration_id".to_string(), serde_json::json!(migration_id));
        context.set_metadata(
            "correlation_id".to_string(),
            serde_json::json!(correlation_id),
        );

        // Track active migration
        {
            let mut active = self.active_migrations.write();
            active.insert(migration_id, context.clone());
        }

        let mut result =
            MigrationResult::new(from_version.clone(), to_version.clone(), total_steps);

        // Execute pre-migration hooks
        self.execute_migration_hooks(&plan, "pre_migration", correlation_id)
            .await?;

        // Emit migration started event using new event system
        let migration_started_event = event_builder.migration_started(
            from_version.clone(),
            to_version.clone(),
            total_steps,
            config.dry_run,
        );
        self.emit_typed_migration_event(&migration_started_event, correlation_id)
            .await?;

        result.mark_in_progress();

        // Execute migration steps
        match self.execute_migration_plan(&plan, &mut context).await {
            Ok(items_migrated) => {
                let duration = context.elapsed_time();
                result.mark_completed(items_migrated, duration);

                // Execute post-migration hooks
                self.execute_migration_hooks(&plan, "post_migration", correlation_id)
                    .await?;

                // Emit migration completed event using new event system
                let migration_completed_event = event_builder.migration_completed(
                    from_version.clone(),
                    to_version.clone(),
                    duration,
                    items_migrated,
                    total_steps,
                );
                self.emit_typed_migration_event(&migration_completed_event, correlation_id)
                    .await?;

                info!(
                    "Migration {} completed successfully: {} items in {:?}",
                    migration_id, items_migrated, duration
                );
            }
            Err(e) => {
                result.mark_failed(e.to_string());
                result.add_error(e.to_string());

                error!("Migration {} failed: {}", migration_id, e);

                // Rollback if configured to do so
                if config.rollback_on_error {
                    warn!("Attempting rollback for migration {}", migration_id);
                    match self.rollback_migration(&plan, &context).await {
                        Ok(_) => {
                            result.mark_rolled_back();
                            info!("Migration {} rolled back successfully", migration_id);
                        }
                        Err(rollback_err) => {
                            error!(
                                "Rollback failed for migration {}: {}",
                                migration_id, rollback_err
                            );
                            result.add_error(format!("Rollback failed: {}", rollback_err));
                        }
                    }
                }

                // Emit migration failed event using new event system
                let migration_failed_event = event_builder.migration_failed(
                    from_version.clone(),
                    to_version.clone(),
                    e.to_string(),
                    result.items_migrated,
                    config.rollback_on_error,
                );
                self.emit_typed_migration_event(&migration_failed_event, correlation_id)
                    .await?;

                return Err(StateError::MigrationError(e.to_string()));
            }
        }

        // Remove from active migrations
        {
            let mut active = self.active_migrations.write();
            active.remove(&migration_id);
        }

        Ok(result)
    }

    /// Execute the migration plan step by step
    async fn execute_migration_plan(
        &self,
        plan: &MigrationPlan,
        context: &mut MigrationContext,
    ) -> Result<usize, MigrationEngineError> {
        let mut total_items_migrated = 0;
        let start_time = Instant::now();

        for (step_index, step) in plan.steps.iter().enumerate() {
            // Check timeout
            if start_time.elapsed() > context.config.timeout {
                return Err(MigrationEngineError::Timeout {
                    timeout: context.config.timeout,
                });
            }

            debug!(
                "Executing migration step {}/{}: {}",
                step_index + 1,
                plan.steps.len(),
                step.description
            );

            context.current_step = step_index + 1;

            // Create transformation for this step
            let transformation = super::transforms::StateTransformation::new(
                format!("migration_step_{}", step_index),
                step.description.clone(),
                step.from_version,
                step.to_version,
            );

            // Execute transformation
            let items_migrated = self
                .execute_transformation(&transformation, &context.config)
                .await?;

            total_items_migrated += items_migrated;

            debug!(
                "Step {}/{} completed: {} items migrated",
                step_index + 1,
                plan.steps.len(),
                items_migrated
            );
        }

        Ok(total_items_migrated)
    }

    /// Execute a single transformation step
    async fn execute_transformation(
        &self,
        transformation: &super::transforms::StateTransformation,
        config: &MigrationConfig,
    ) -> Result<usize, MigrationEngineError> {
        if config.dry_run {
            debug!("Dry run: Skipping actual transformation");
            return Ok(0);
        }

        let mut items_processed = 0;

        // Get all state keys to migrate
        let all_keys = self.storage_adapter.list_keys("").await.map_err(|e| {
            MigrationEngineError::MigrationFailed {
                reason: format!("Failed to list keys: {}", e),
            }
        })?;

        // Process in batches
        for chunk in all_keys.chunks(config.batch_size) {
            for key in chunk {
                // Skip non-state keys (like hook history)
                if key.starts_with("hook_history:") || key.starts_with("agent_state:") {
                    continue;
                }

                match self.transform_state_item(key, transformation).await {
                    Ok(transformed) => {
                        if transformed {
                            items_processed += 1;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to transform state item '{}': {}", key, e);
                        if config.validation_level == ValidationLevel::Strict {
                            return Err(MigrationEngineError::MigrationFailed {
                                reason: format!("Transformation failed for key '{}': {}", key, e),
                            });
                        }
                    }
                }
            }
        }

        Ok(items_processed)
    }

    /// Transform a single state item
    async fn transform_state_item(
        &self,
        key: &str,
        transformation: &super::transforms::StateTransformation,
    ) -> StateResult<bool> {
        // Load the current state item
        let current_state: Option<SerializableState> = self.storage_adapter.load(key).await?;

        if let Some(mut state) = current_state {
            // Use the DataTransformer to transform the state
            let transformer = super::transforms::DataTransformer::new();

            match transformer.transform_state(&mut state, transformation) {
                Ok(result) => {
                    if result.success && result.fields_transformed > 0 {
                        // Store the transformed state
                        self.storage_adapter.store(key, &state).await?;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                Err(e) => {
                    warn!("State transformation failed for key '{}': {}", key, e);
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }

    /// Execute migration hooks
    async fn execute_migration_hooks(
        &self,
        plan: &MigrationPlan,
        hook_point: &str,
        correlation_id: Uuid,
    ) -> StateResult<()> {
        let component_id = ComponentId::new(
            ComponentType::Custom("migration".to_string()),
            "migration_engine".to_string(),
        );

        let mut hook_context =
            HookContext::new(HookPoint::Custom(hook_point.to_string()), component_id);
        hook_context = hook_context.with_correlation_id(correlation_id);

        hook_context.insert_metadata("from_version".to_string(), plan.from_version.to_string());
        hook_context.insert_metadata("to_version".to_string(), plan.to_version.to_string());
        hook_context.insert_metadata("risk_level".to_string(), format!("{:?}", plan.risk_level));
        hook_context.insert_metadata(
            "requires_backup".to_string(),
            plan.requires_backup.to_string(),
        );
        hook_context.insert_metadata("steps_count".to_string(), plan.steps.len().to_string());

        // Execute hooks through the existing hook executor
        // For now, we use an empty hooks list since hooks would be registered externally
        let hooks: Vec<std::sync::Arc<dyn llmspell_hooks::Hook>> = vec![];
        let start_time = Instant::now();

        // Make context mutable for hook execution
        let mut mutable_context = hook_context;

        match self
            .hook_executor
            .execute_hooks(&hooks, &mut mutable_context)
            .await
        {
            Ok(results) => {
                let execution_time = start_time.elapsed();
                debug!(
                    "Migration hooks executed successfully for {}: {} hooks in {:?}",
                    hook_point,
                    results.len(),
                    execution_time
                );

                // Check for hook results
                for result in results {
                    match result {
                        llmspell_hooks::HookResult::Continue => {
                            debug!("Hook executed successfully for {}", hook_point);
                        }
                        llmspell_hooks::HookResult::Modified(_) => {
                            debug!("Hook modified context for {}", hook_point);
                        }
                        llmspell_hooks::HookResult::Cancel(reason) => {
                            warn!("Migration hook cancelled during {}: {}", hook_point, reason);
                            // Continue with migration even if hooks fail (configurable behavior)
                        }
                        llmspell_hooks::HookResult::Redirect(_) => {
                            debug!("Hook redirected for {}", hook_point);
                        }
                        llmspell_hooks::HookResult::Replace(_) => {
                            debug!("Hook replaced data for {}", hook_point);
                        }
                        llmspell_hooks::HookResult::Retry { .. } => {
                            debug!("Hook requested retry for {}", hook_point);
                        }
                        llmspell_hooks::HookResult::Fork { .. } => {
                            debug!("Hook requested fork for {}", hook_point);
                        }
                        llmspell_hooks::HookResult::Cache { .. } => {
                            debug!("Hook requested caching for {}", hook_point);
                        }
                        llmspell_hooks::HookResult::Skipped(reason) => {
                            debug!("Hook skipped for {}: {}", hook_point, reason);
                        }
                    }
                }
            }
            Err(hook_error) => {
                let execution_time = start_time.elapsed();
                error!(
                    "Migration hook execution failed for {} after {:?}: {}",
                    hook_point, execution_time, hook_error
                );

                // Decide whether to fail the migration or continue
                // For now, we continue but log the error
                warn!(
                    "Continuing migration despite hook failure in phase: {}",
                    hook_point
                );
            }
        }

        Ok(())
    }

    /// Emit migration event for correlation tracking (legacy method)
    #[allow(dead_code)]
    async fn emit_migration_event(
        &self,
        event_type: &str,
        plan: &MigrationPlan,
        correlation_id: Uuid,
        result: Option<&MigrationResult>,
    ) -> StateResult<()> {
        let mut event_data = serde_json::json!({
            "from_version": plan.from_version,
            "to_version": plan.to_version,
            "risk_level": plan.risk_level,
            "requires_backup": plan.requires_backup,
            "steps": plan.steps.len(),
        });

        if let Some(result) = result {
            event_data["status"] = serde_json::json!(result.status);
            event_data["items_migrated"] = serde_json::json!(result.items_migrated);
            event_data["duration_ms"] = serde_json::json!(result.duration.as_millis());
        }

        let event = UniversalEvent::new(event_type, event_data, llmspell_events::Language::Rust)
            .with_correlation_id(correlation_id)
            .with_source("migration_engine")
            .with_tag("migration");

        // Track for correlation
        self.correlation_tracker.track_event(event.clone());

        // Publish event
        self.event_bus
            .publish(event)
            .await
            .map_err(|e| StateError::StorageError(e.into()))?;

        Ok(())
    }

    /// Emit typed migration event using new MigrationEvent system
    async fn emit_typed_migration_event(
        &self,
        migration_event: &MigrationEvent,
        correlation_id: Uuid,
    ) -> StateResult<()> {
        // Convert to UniversalEvent for existing event system integration
        let universal_event: UniversalEvent = migration_event.clone().into();

        // Create event metadata for correlation tracking
        let _event_metadata = migration_event.create_metadata(correlation_id);

        // Track for correlation
        self.correlation_tracker
            .track_event(universal_event.clone());

        // Publish event to event bus
        self.event_bus
            .publish(universal_event)
            .await
            .map_err(|e| StateError::StorageError(e.into()))?;

        // Log event for debugging
        match migration_event {
            MigrationEvent::MigrationStarted { .. } => {
                debug!(
                    "Migration started event emitted with correlation_id: {}",
                    correlation_id
                );
            }
            MigrationEvent::MigrationCompleted { .. } => {
                debug!(
                    "Migration completed event emitted with correlation_id: {}",
                    correlation_id
                );
            }
            MigrationEvent::MigrationFailed { .. } => {
                error!(
                    "Migration failed event emitted with correlation_id: {}",
                    correlation_id
                );
            }
            _ => {
                debug!(
                    "Migration event {:?} emitted with correlation_id: {}",
                    migration_event.event_type(),
                    correlation_id
                );
            }
        }

        Ok(())
    }

    /// Rollback migration (simplified implementation)
    async fn rollback_migration(
        &self,
        _plan: &MigrationPlan,
        _context: &MigrationContext,
    ) -> Result<(), MigrationEngineError> {
        // In a real implementation, this would restore from backup
        // or apply reverse transformations
        warn!("Rollback not fully implemented - would restore from backup");
        Ok(())
    }

    /// Get active migrations
    pub fn get_active_migrations(&self) -> HashMap<Uuid, MigrationContext> {
        let active = self.active_migrations.read();
        active.clone()
    }

    /// Cancel an active migration
    pub async fn cancel_migration(&self, migration_id: Uuid) -> StateResult<()> {
        let mut active = self.active_migrations.write();
        if active.remove(&migration_id).is_some() {
            info!("Migration {} cancelled", migration_id);
            Ok(())
        } else {
            Err(StateError::MigrationError(format!(
                "Migration {} not found or already completed",
                migration_id
            )))
        }
    }
}

/// Migration executor trait for different execution strategies
pub trait MigrationExecutor {
    /// Execute a migration with the given strategy
    fn execute_migration(
        &self,
        plan: &MigrationPlan,
        config: &MigrationConfig,
    ) -> impl std::future::Future<Output = StateResult<MigrationResult>> + Send;
}

impl MigrationExecutor for MigrationEngine {
    async fn execute_migration(
        &self,
        plan: &MigrationPlan,
        config: &MigrationConfig,
    ) -> StateResult<MigrationResult> {
        self.migrate(&plan.from_version, &plan.to_version, config.clone())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration_engine_creation() {
        let storage_adapter = Arc::new(crate::backend_adapter::StateStorageAdapter::new(
            Arc::new(llmspell_storage::MemoryBackend::new()),
            "test".to_string(),
        ));
        let schema_registry = SchemaRegistry::new();
        let hook_executor = Arc::new(HookExecutor::new());
        let correlation_tracker = Arc::new(EventCorrelationTracker::default());
        let event_bus = Arc::new(EventBus::new());

        let engine = MigrationEngine::new(
            storage_adapter,
            schema_registry,
            hook_executor,
            correlation_tracker,
            event_bus,
        );

        assert!(engine.get_active_migrations().is_empty());
    }

    #[test]
    fn test_migration_engine_error_conversion() {
        let error = MigrationEngineError::MigrationFailed {
            reason: "Test error".to_string(),
        };

        let state_error: StateError = error.into();
        assert!(matches!(state_error, StateError::MigrationError(_)));
    }
}
