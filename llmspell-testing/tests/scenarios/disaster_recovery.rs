// ABOUTME: Disaster recovery scenario tests for operational readiness validation
// ABOUTME: Tests complete system failure and recovery procedures including data consistency

use llmspell_kernel::state::{
    backup::{BackupConfig, BackupManager, CompressionType, RestoreOptions},
    config::{PersistenceConfig, StorageBackendType},
    manager::StateManager,
    StateScope,
};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;
use tokio::time::sleep;

#[cfg(test)]
mod disaster_recovery_scenarios {
    use super::*;

    /// Represents a simulated application with critical components
    struct TestApplication {
        state_manager: Arc<StateManager>,
        backup_manager: Arc<BackupManager>,
        _temp_dir: TempDir,
    }

    impl TestApplication {
        async fn new() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let backup_dir = temp_dir.path().join("backups");
            std::fs::create_dir_all(&backup_dir).unwrap();

            let persistence_config = PersistenceConfig {
                enabled: true,
                ..Default::default()
            };

            let backup_config = BackupConfig {
                backup_dir: backup_dir.clone(),
                compression_enabled: true,
                compression_type: CompressionType::Zstd,
                compression_level: 3,
                encryption_enabled: false,
                max_backups: Some(10),
                incremental_enabled: true,
                max_backup_age: Some(Duration::from_secs(30 * 24 * 60 * 60)), // 30 days
                ..Default::default()
            };

            let state_manager = Arc::new(
                StateManager::with_backend(StorageBackendType::Memory, persistence_config, None)
                    .await
                    .unwrap(),
            );

            let backup_manager =
                Arc::new(BackupManager::new(backup_config, state_manager.clone()).unwrap());

            Self {
                state_manager,
                backup_manager,
                _temp_dir: temp_dir,
            }
        }

        /// Initialize application with critical system state
        async fn initialize_critical_state(&self) -> Result<(), Box<dyn std::error::Error>> {
            // Database configuration
            self.state_manager
                .set(
                    StateScope::Global,
                    "database_config",
                    json!({
                        "host": "db.production.com",
                        "port": 5432,
                        "database": "production_db",
                        "connection_pool_size": 20,
                        "timeout_seconds": 30
                    }),
                )
                .await?;

            // Service registry
            self.state_manager.set(StateScope::Global, "service_registry", json!({
                "services": [
                    {"name": "auth-service", "url": "https://auth.prod.com", "health": "healthy"},
                    {"name": "user-service", "url": "https://users.prod.com", "health": "healthy"},
                    {"name": "billing-service", "url": "https://billing.prod.com", "health": "healthy"}
                ],
                "load_balancer": {
                    "algorithm": "round_robin",
                    "health_check_interval": 30
                }
            })).await?;

            // Feature flags
            self.state_manager
                .set(
                    StateScope::Global,
                    "feature_flags",
                    json!({
                        "new_checkout_flow": true,
                        "advanced_analytics": false,
                        "beta_features": false,
                        "maintenance_mode": false,
                        "emergency_shutdown": false
                    }),
                )
                .await?;

            // Agent configurations
            for i in 1..=5 {
                self.state_manager
                    .set(
                        StateScope::Custom(format!("agent_{}", i)),
                        "config",
                        json!({
                            "id": i,
                            "type": "customer_service",
                            "model": "gpt-4",
                            "temperature": 0.7,
                            "max_tokens": 2000,
                            "system_prompt": "You are a helpful customer service agent.",
                            "active": true,
                            "last_health_check": "2025-01-27T10:00:00Z"
                        }),
                    )
                    .await?;

                // Agent conversation history
                self.state_manager
                    .set(
                        StateScope::Custom(format!("agent_{}", i)),
                        "history",
                        json!({
                            "conversations": [],
                            "total_interactions": 0,
                            "avg_response_time": 0.0,
                            "customer_satisfaction": 4.5
                        }),
                    )
                    .await?;
            }

            // User session data
            for i in 1..=10 {
                self.state_manager
                    .set(
                        StateScope::Custom(format!("user_session_{}", i)),
                        "data",
                        json!({
                            "user_id": 1000 + i,
                            "session_start": "2025-01-27T09:00:00Z",
                            "permissions": ["read", "write"],
                            "shopping_cart": [
                                {"item_id": i * 10, "quantity": 2, "price": 29.99},
                                {"item_id": i * 10 + 1, "quantity": 1, "price": 49.99}
                            ],
                            "total_value": 109.97
                        }),
                    )
                    .await?;
            }

            Ok(())
        }

        /// Simulate complete system failure
        async fn simulate_disaster(&self) {
            // Clear all state to simulate complete system failure
            // We need to clear all scopes that contain data

            // Clear global scope
            self.state_manager
                .clear_scope(StateScope::Global)
                .await
                .unwrap();

            // Clear agent scopes
            for i in 1..=5 {
                self.state_manager
                    .clear_scope(StateScope::Custom(format!("agent_{}", i)))
                    .await
                    .unwrap();
            }

            // Clear user session scopes
            for i in 1..=10 {
                self.state_manager
                    .clear_scope(StateScope::Custom(format!("user_session_{}", i)))
                    .await
                    .unwrap();
            }
        }

        /// Verify system integrity after recovery
        async fn verify_system_integrity(&self) -> Result<bool, Box<dyn std::error::Error>> {
            // Check critical configuration
            let db_config = self
                .state_manager
                .get(StateScope::Global, "database_config")
                .await?;
            if db_config.is_none() {
                return Ok(false);
            }

            let service_registry = self
                .state_manager
                .get(StateScope::Global, "service_registry")
                .await?;
            if service_registry.is_none() {
                return Ok(false);
            }

            let feature_flags = self
                .state_manager
                .get(StateScope::Global, "feature_flags")
                .await?;
            if feature_flags.is_none() {
                return Ok(false);
            }

            // Check all agent configurations
            for i in 1..=5 {
                let agent_config = self
                    .state_manager
                    .get(StateScope::Custom(format!("agent_{}", i)), "config")
                    .await?;
                if agent_config.is_none() {
                    return Ok(false);
                }

                let agent_history = self
                    .state_manager
                    .get(StateScope::Custom(format!("agent_{}", i)), "history")
                    .await?;
                if agent_history.is_none() {
                    return Ok(false);
                }
            }

            // Check user sessions
            for i in 1..=10 {
                let session_data = self
                    .state_manager
                    .get(StateScope::Custom(format!("user_session_{}", i)), "data")
                    .await?;
                if session_data.is_none() {
                    return Ok(false);
                }
            }

            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_complete_system_disaster_recovery() {
        let app = TestApplication::new().await;

        // Step 1: Initialize critical application state
        app.initialize_critical_state().await.unwrap();

        // Step 2: Create disaster recovery backup
        let disaster_backup = app.backup_manager.create_backup(false).await.unwrap();
        println!(
            "🔍 Backup created with {} entries (expected >= 20)",
            disaster_backup.entry_count
        );

        // Debug: Let's see what's actually in the state before backup
        let global_keys = app
            .state_manager
            .list_keys(StateScope::Global)
            .await
            .unwrap();
        println!("🔍 Global scope keys: {:?}", global_keys);

        // Check agent scopes
        for i in 1..=5 {
            let agent_scope = StateScope::Custom(format!("agent_{}", i));
            let agent_keys = app.state_manager.list_keys(agent_scope).await.unwrap();
            println!("🔍 Agent {} scope keys: {:?}", i, agent_keys);
        }

        // Check user session scopes
        for i in 1..=3 {
            // Just check first 3 to avoid spam
            let session_scope = StateScope::Custom(format!("user_session_{}", i));
            let session_keys = app.state_manager.list_keys(session_scope).await.unwrap();
            println!("🔍 User session {} scope keys: {:?}", i, session_keys);
        }

        assert!(
            disaster_backup.entry_count >= 20,
            "Should have all our test data, got {} entries",
            disaster_backup.entry_count
        );

        // Step 3: Validate backup before disaster
        let validation = app
            .backup_manager
            .validate_backup(&disaster_backup.id)
            .await
            .unwrap();
        assert!(validation.is_valid);
        assert!(validation.checksum_valid);
        assert!(validation.integrity_valid);

        // Step 4: Simulate complete system disaster
        app.simulate_disaster().await;

        // Step 5: Verify system is down (no state exists)
        let db_config = app
            .state_manager
            .get(StateScope::Global, "database_config")
            .await
            .unwrap();
        assert_eq!(db_config, None);

        // Step 6: Begin disaster recovery
        let recovery_start = SystemTime::now();

        let restore_result = app
            .backup_manager
            .restore_backup(
                &disaster_backup.id,
                RestoreOptions {
                    verify_checksums: true,
                    backup_current: false, // No need to backup empty state
                    target_version: None,
                    dry_run: false,
                },
            )
            .await;

        assert!(restore_result.is_ok(), "Disaster recovery should succeed");

        let recovery_duration = recovery_start.elapsed().unwrap();

        // Step 7: Verify complete system recovery
        let integrity_check = app.verify_system_integrity().await.unwrap();
        assert!(integrity_check, "System integrity should be fully restored");

        // Step 8: Verify specific critical components
        let db_config = app
            .state_manager
            .get(StateScope::Global, "database_config")
            .await
            .unwrap();
        assert!(db_config.is_some());
        let db_config_value = db_config.unwrap();
        assert_eq!(db_config_value["host"], "db.production.com");
        assert_eq!(db_config_value["port"], 5432);

        let feature_flags = app
            .state_manager
            .get(StateScope::Global, "feature_flags")
            .await
            .unwrap();
        assert!(feature_flags.is_some());
        let flags_value = feature_flags.unwrap();
        assert_eq!(flags_value["maintenance_mode"], false);
        assert_eq!(flags_value["new_checkout_flow"], true);

        // Step 9: Verify agent functionality
        for i in 1..=5 {
            let agent_config = app
                .state_manager
                .get(StateScope::Custom(format!("agent_{}", i)), "config")
                .await
                .unwrap();
            assert!(agent_config.is_some());
            let config_value = agent_config.unwrap();
            assert_eq!(config_value["active"], true);
            assert_eq!(config_value["type"], "customer_service");
        }

        // Step 10: Performance validation
        assert!(
            recovery_duration < Duration::from_secs(30),
            "Disaster recovery should complete within 30 seconds, took {:?}",
            recovery_duration
        );

        println!("✅ Complete system disaster recovery successful!");
        println!("   Recovery time: {:?}", recovery_duration);
        println!("   Entries restored: {}", disaster_backup.entry_count);
        println!("   Backup size: {} bytes", disaster_backup.size_bytes);
    }

    #[tokio::test]
    async fn test_partial_system_failure_recovery() {
        let app = TestApplication::new().await;

        // Initialize system
        app.initialize_critical_state().await.unwrap();

        // Create backup point
        let backup = app.backup_manager.create_backup(false).await.unwrap();

        // Simulate partial failure - corrupt agent data only
        for i in 1..=5 {
            app.state_manager
                .delete(StateScope::Custom(format!("agent_{}", i)), "config")
                .await
                .unwrap();

            app.state_manager
                .delete(StateScope::Custom(format!("agent_{}", i)), "history")
                .await
                .unwrap();
        }

        // Verify partial failure
        let db_config = app
            .state_manager
            .get(StateScope::Global, "database_config")
            .await
            .unwrap();
        assert!(db_config.is_some(), "Global config should still exist");

        let agent_1_config = app
            .state_manager
            .get(StateScope::Custom("agent_1".to_string()), "config")
            .await
            .unwrap();
        assert_eq!(agent_1_config, None, "Agent config should be gone");

        // Perform selective recovery
        app.backup_manager
            .restore_backup(&backup.id, RestoreOptions::default())
            .await
            .unwrap();

        // Verify full recovery
        let integrity_check = app.verify_system_integrity().await.unwrap();
        assert!(integrity_check, "System should be fully recovered");

        println!("✅ Partial system failure recovery successful!");
    }

    #[tokio::test]
    async fn test_point_in_time_recovery() {
        let app = TestApplication::new().await;

        // Time T0: Initial state
        app.initialize_critical_state().await.unwrap();
        let t0_backup = app.backup_manager.create_backup(false).await.unwrap();

        sleep(Duration::from_millis(50)).await;

        // Time T1: Add more data
        app.state_manager
            .set(StateScope::Global, "t1_data", json!({"timestamp": "T1"}))
            .await
            .unwrap();
        let t1_backup = app.backup_manager.create_backup(true).await.unwrap();

        sleep(Duration::from_millis(50)).await;

        // Time T2: Modify existing data
        app.state_manager
            .set(
                StateScope::Global,
                "feature_flags",
                json!({
                    "new_checkout_flow": false,  // Changed
                    "advanced_analytics": true,  // Changed
                    "beta_features": true,       // Changed
                    "maintenance_mode": false,
                    "emergency_shutdown": false
                }),
            )
            .await
            .unwrap();
        let t2_backup = app.backup_manager.create_backup(true).await.unwrap();

        sleep(Duration::from_millis(50)).await;

        // Time T3: Add critical error state
        app.state_manager
            .set(
                StateScope::Global,
                "system_errors",
                json!({
                    "critical_error": true,
                    "error_message": "Database connection failed",
                    "occurred_at": "T3"
                }),
            )
            .await
            .unwrap();
        let t3_backup = app.backup_manager.create_backup(true).await.unwrap();

        // Scenario: Recover to T1 (before the problematic changes at T2/T3)
        app.backup_manager
            .restore_backup(&t1_backup.id, RestoreOptions::default())
            .await
            .unwrap();

        // Verify T1 state
        let t1_data = app
            .state_manager
            .get(StateScope::Global, "t1_data")
            .await
            .unwrap();
        assert_eq!(t1_data, Some(json!({"timestamp": "T1"})));

        let feature_flags = app
            .state_manager
            .get(StateScope::Global, "feature_flags")
            .await
            .unwrap();
        let flags_value = feature_flags.unwrap();
        assert_eq!(flags_value["new_checkout_flow"], true); // Original value

        let system_errors = app
            .state_manager
            .get(StateScope::Global, "system_errors")
            .await
            .unwrap();
        assert_eq!(system_errors, None); // Should not exist at T1

        // Verify we can also recover to T2 if needed
        app.backup_manager
            .restore_backup(&t2_backup.id, RestoreOptions::default())
            .await
            .unwrap();

        let feature_flags_t2 = app
            .state_manager
            .get(StateScope::Global, "feature_flags")
            .await
            .unwrap();
        let flags_t2_value = feature_flags_t2.unwrap();
        assert_eq!(flags_t2_value["new_checkout_flow"], false); // T2 value
        assert_eq!(flags_t2_value["advanced_analytics"], true); // T2 value

        let system_errors_t2 = app
            .state_manager
            .get(StateScope::Global, "system_errors")
            .await
            .unwrap();
        assert_eq!(system_errors_t2, None); // Still should not exist at T2

        println!("✅ Point-in-time recovery successful!");
        println!("   T0 backup: {}", t0_backup.id);
        println!("   T1 backup: {}", t1_backup.id);
        println!("   T2 backup: {}", t2_backup.id);
        println!("   T3 backup: {}", t3_backup.id);
    }

    #[tokio::test]
    async fn test_recovery_under_load() {
        let app = TestApplication::new().await;

        // Create a large dataset
        for i in 0..500 {
            app.state_manager.set(
                StateScope::Global,
                &format!("load_test_item_{}", i),
                json!({
                    "id": i,
                    "data": format!("Large data string for load test item {} with extensive content to simulate real-world data size and complexity. This helps test recovery performance under realistic conditions.", i),
                    "metadata": {
                        "created_at": "2025-01-27T10:00:00Z",
                        "size": "large",
                        "category": format!("category_{}", i % 10),
                        "priority": i % 5,
                        "tags": vec![format!("tag_{}", i % 3), format!("tag_{}", i % 7)]
                    }
                })
            ).await.unwrap();
        }

        // Create backup of large dataset
        let load_backup = app.backup_manager.create_backup(false).await.unwrap();
        assert_eq!(load_backup.entry_count, 500);

        // Simulate system failure
        app.simulate_disaster().await;

        // Perform recovery under simulated load conditions
        let recovery_start = SystemTime::now();

        // Start recovery
        let recovery_task = {
            let backup_manager = app.backup_manager.clone();
            let backup_id = load_backup.id.clone();
            tokio::spawn(async move {
                backup_manager
                    .restore_backup(&backup_id, RestoreOptions::default())
                    .await
            })
        };

        // Simulate concurrent system operations during recovery
        let monitoring_task = {
            let state_manager = app.state_manager.clone();
            tokio::spawn(async move {
                for _ in 0..10 {
                    sleep(Duration::from_millis(100)).await;
                    // Try to read data during recovery (should handle gracefully)
                    let _ = state_manager
                        .get(StateScope::Global, "load_test_item_0")
                        .await;
                }
            })
        };

        // Wait for both tasks
        let (recovery_result, _) = tokio::join!(recovery_task, monitoring_task);
        let recovery_duration = recovery_start.elapsed().unwrap();

        // Verify recovery succeeded
        assert!(
            recovery_result.unwrap().is_ok(),
            "Recovery should succeed under load"
        );

        // Verify data integrity
        for i in [0, 100, 250, 499] {
            // Spot check various items
            let item = app
                .state_manager
                .get(StateScope::Global, &format!("load_test_item_{}", i))
                .await
                .unwrap();
            assert!(item.is_some(), "Item {} should be recovered", i);
            let item_value = item.unwrap();
            assert_eq!(item_value["id"], i);
        }

        // Performance validation
        assert!(
            recovery_duration < Duration::from_secs(60),
            "Large dataset recovery should complete within 60 seconds, took {:?}",
            recovery_duration
        );

        println!("✅ Recovery under load successful!");
        println!("   Items recovered: {}", load_backup.entry_count);
        println!("   Recovery time: {:?}", recovery_duration);
        println!("   Backup size: {} bytes", load_backup.size_bytes);
        println!(
            "   Throughput: {:.2} items/sec",
            load_backup.entry_count as f64 / recovery_duration.as_secs_f64()
        );
    }

    #[tokio::test]
    async fn test_cascading_failure_recovery() {
        let app = TestApplication::new().await;

        // Initialize system
        app.initialize_critical_state().await.unwrap();

        // Create initial backup
        let stable_backup = app.backup_manager.create_backup(false).await.unwrap();

        // Simulate cascading failure scenario
        // 1. First failure: Agent 1 goes down
        app.state_manager
            .set(
                StateScope::Custom("agent_1".to_string()),
                "config",
                json!({"active": false, "error": "Connection timeout"}),
            )
            .await
            .unwrap();

        // 2. Second failure: Database connection issues
        app.state_manager
            .set(
                StateScope::Global,
                "database_config",
                json!({
                    "host": "db.production.com",
                    "port": 5432,
                    "database": "production_db",
                    "connection_pool_size": 5, // Reduced
                    "timeout_seconds": 5,      // Reduced
                    "status": "degraded"       // Added error status
                }),
            )
            .await
            .unwrap();

        // 3. Third failure: Emergency shutdown triggered
        app.state_manager
            .set(
                StateScope::Global,
                "feature_flags",
                json!({
                    "new_checkout_flow": false,
                    "advanced_analytics": false,
                    "beta_features": false,
                    "maintenance_mode": true,      // Emergency mode
                    "emergency_shutdown": true     // System shutdown
                }),
            )
            .await
            .unwrap();

        // 4. Complete system collapse
        sleep(Duration::from_millis(100)).await;
        app.simulate_disaster().await;

        // Attempt recovery to pre-failure state
        let recovery_start = SystemTime::now();
        app.backup_manager
            .restore_backup(&stable_backup.id, RestoreOptions::default())
            .await
            .unwrap();
        let recovery_duration = recovery_start.elapsed().unwrap();

        // Verify complete recovery to stable state
        let db_config = app
            .state_manager
            .get(StateScope::Global, "database_config")
            .await
            .unwrap();
        let db_value = db_config.unwrap();
        assert_eq!(db_value["connection_pool_size"], 20); // Original value
        assert_eq!(db_value["timeout_seconds"], 30); // Original value
        assert!(!matches!(db_value.get("status"), Some(v) if v == "degraded"));

        let feature_flags = app
            .state_manager
            .get(StateScope::Global, "feature_flags")
            .await
            .unwrap();
        let flags_value = feature_flags.unwrap();
        assert_eq!(flags_value["maintenance_mode"], false); // Original value
        assert_eq!(flags_value["emergency_shutdown"], false); // Original value
        assert_eq!(flags_value["new_checkout_flow"], true); // Original value

        let agent_1_config = app
            .state_manager
            .get(StateScope::Custom("agent_1".to_string()), "config")
            .await
            .unwrap();
        let agent_1_value = agent_1_config.unwrap();
        assert_eq!(agent_1_value["active"], true); // Original value
        assert!(agent_1_value.get("error").is_none());

        // Verify system integrity
        let integrity_check = app.verify_system_integrity().await.unwrap();
        assert!(
            integrity_check,
            "System should be fully recovered from cascading failure"
        );

        assert!(
            recovery_duration < Duration::from_secs(15),
            "Cascading failure recovery should be quick, took {:?}",
            recovery_duration
        );

        println!("✅ Cascading failure recovery successful!");
        println!("   Recovery time: {:?}", recovery_duration);
        println!("   System restored to stable state");
    }
}
