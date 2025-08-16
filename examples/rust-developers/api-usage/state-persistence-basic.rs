// ABOUTME: Basic state operations in Rust demonstrating StateManager API
// ABOUTME: Shows how to create, configure, and use StateManager with different backends

//! # Basic State Operations Example
//! 
//! This example demonstrates the fundamental state persistence operations:
//! - Creating a StateManager with different backends
//! - Basic CRUD operations (Create, Read, Update, Delete)
//! - Working with different state scopes
//! - Error handling patterns
//! - Performance considerations
//! 
//! ## Running this example
//! 
//! ```bash
//! cargo run --example basic_operations
//! ```

use llmspell_state_persistence::{
    StateManager, StateScope, PersistenceConfig, StorageBackendType, SledConfig
};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🗄️  State Persistence - Basic Operations Example (Rust)");
    println!("======================================================");
    println!("This example demonstrates StateManager usage in Rust\n");
    
    // 1. CREATE STATE MANAGER WITH MEMORY BACKEND
    println!("1. Creating StateManager with memory backend...");
    let memory_state_manager = StateManager::new().await?;
    println!("   ✅ Memory-based StateManager created");
    
    // 2. BASIC OPERATIONS
    println!("\n2. Basic state operations...");
    
    // Set operation
    let start = Instant::now();
    memory_state_manager.set(
        StateScope::Global,
        "app_config",
        json!({
            "name": "rs-llmspell",
            "version": "0.4.0",
            "features": {
                "state_persistence": true,
                "hooks": true,
                "migrations": true
            },
            "performance": {
                "max_cache_size": 100_000_000,
                "fast_path_enabled": true
            }
        })
    ).await?;
    let set_duration = start.elapsed();
    println!("   ✅ Set operation completed in {:?}", set_duration);
    
    // Get operation
    let start = Instant::now(); 
    let config = memory_state_manager.get(StateScope::Global, "app_config").await?;
    let get_duration = start.elapsed();
    println!("   ✅ Get operation completed in {:?}", get_duration);
    
    if let Some(config_value) = config {
        println!("   📄 Retrieved config:");
        println!("      - Name: {}", config_value["name"]);
        println!("      - Version: {}", config_value["version"]);
        println!("      - State persistence: {}", config_value["features"]["state_persistence"]);
    }
    
    // 3. DIFFERENT SCOPES
    println!("\n3. Working with different scopes...");
    
    // Global scope
    memory_state_manager.set(
        StateScope::Global,
        "system_status",
        json!("running")
    ).await?;
    
    // Agent scope
    let agent_scope = StateScope::Agent("gpt-4-assistant".to_string());
    memory_state_manager.set(
        agent_scope.clone(),
        "conversation_history",
        json!([
            {
                "role": "user",
                "content": "Hello, how are you?",
                "timestamp": chrono::Utc::now().to_rfc3339()
            },
            {
                "role": "assistant", 
                "content": "Hello! I'm doing well, thank you for asking. How can I help you today?",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        ])
    ).await?;
    
    // Workflow scope
    let workflow_scope = StateScope::Workflow("data-processing".to_string());
    memory_state_manager.set(
        workflow_scope.clone(),
        "execution_state",
        json!({
            "current_step": "validation",
            "completed_steps": ["input", "preprocessing"],
            "total_steps": 5,
            "progress": 0.4,
            "started_at": chrono::Utc::now().to_rfc3339()
        })
    ).await?;
    
    // Step scope
    let step_scope = StateScope::Step {
        workflow_id: "data-processing".to_string(),
        step_name: "validation".to_string(),
    };
    memory_state_manager.set(
        step_scope.clone(),
        "validation_results",
        json!({
            "total_records": 10000,
            "valid_records": 9850,
            "invalid_records": 150,
            "error_details": [
                {"line": 45, "error": "Missing required field 'email'"},
                {"line": 123, "error": "Invalid date format"},
                {"line": 501, "error": "Duplicate ID"}
            ]
        })
    ).await?;
    
    println!("   ✅ Data saved to multiple scopes");
    
    // Retrieve from different scopes
    let system_status = memory_state_manager.get(StateScope::Global, "system_status").await?;
    let conversation = memory_state_manager.get(agent_scope, "conversation_history").await?;
    let workflow_state = memory_state_manager.get(workflow_scope, "execution_state").await?;
    let validation_results = memory_state_manager.get(step_scope, "validation_results").await?;
    
    println!("   📊 Scope data summary:");
    println!("      - System status: {}", system_status.unwrap_or(json!("unknown")));
    
    if let Some(conv) = conversation {
        println!("      - Conversation messages: {}", conv.as_array().unwrap().len());
    }
    
    if let Some(workflow) = workflow_state {
        println!("      - Workflow step: {}", workflow["current_step"]);
        println!("      - Progress: {}%", (workflow["progress"].as_f64().unwrap_or(0.0) * 100.0) as i32);
    }
    
    if let Some(validation) = validation_results {
        println!("      - Valid records: {}/{}", 
                 validation["valid_records"], 
                 validation["total_records"]);
    }
    
    // 4. PERSISTENT STORAGE BACKEND (optional - for demonstration)
    println!("\n4. Creating StateManager with persistent storage...");
    
    // Create temporary directory for this example
    let temp_dir = tempfile::TempDir::new()?;
    let persistent_state_manager = StateManager::with_backend(
        StorageBackendType::Sled(SledConfig {
            path: temp_dir.path().join("example_state"),
            cache_capacity: 10 * 1024 * 1024, // 10MB
            use_compression: true,
        }),
        PersistenceConfig::default(),
    ).await?;
    
    println!("   ✅ Persistent StateManager created at {:?}", temp_dir.path());
    
    // Save data to persistent storage
    persistent_state_manager.set(
        StateScope::Global,
        "persistent_example",
        json!({
            "message": "This data survives application restarts!",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "example_id": "basic_operations_rust"
        })
    ).await?;
    
    println!("   ✅ Data saved to persistent storage");
    
    // 5. LIST KEYS (if supported)
    println!("\n5. Listing keys in scopes...");
    
    match memory_state_manager.list_keys(StateScope::Global).await {
        Ok(keys) => {
            println!("   📋 Global scope keys: {:?}", keys);
        }
        Err(e) => {
            println!("   ⚠️  Could not list keys: {}", e);
        }
    }
    
    // 6. DELETE OPERATIONS
    println!("\n6. Delete operations...");
    
    let deleted = memory_state_manager.delete(StateScope::Global, "system_status").await?;
    println!("   🗑️  Deleted 'system_status': {}", deleted);
    
    // Verify deletion
    let deleted_value = memory_state_manager.get(StateScope::Global, "system_status").await?;
    println!("   ✅ Verification - deleted value is: {:?}", deleted_value);
    
    // 7. ERROR HANDLING
    println!("\n7. Error handling examples...");
    
    // Try to get non-existent key
    let missing = memory_state_manager.get(StateScope::Global, "non_existent_key").await?;
    println!("   📭 Non-existent key returns: {:?}", missing);
    
    // Handle potential errors gracefully
    match memory_state_manager.set(
        StateScope::Global,
        "test_key",
        json!("test_value")
    ).await {
        Ok(()) => println!("   ✅ Set operation succeeded"),
        Err(e) => println!("   ❌ Set operation failed: {}", e),
    }
    
    // 8. PERFORMANCE MEASUREMENT
    println!("\n8. Performance measurement...");
    
    let iterations = 1000;
    let start = Instant::now();
    
    for i in 0..iterations {
        let key = format!("perf_test_{}", i);
        let value = json!({
            "id": i,
            "data": format!("test_data_{}", i),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        memory_state_manager.set(StateScope::Global, &key, value).await?;
    }
    
    let write_duration = start.elapsed();
    let writes_per_second = iterations as f64 / write_duration.as_secs_f64();
    
    println!("   📊 Performance results:");
    println!("      - {} write operations in {:?}", iterations, write_duration);
    println!("      - {:.0} writes per second", writes_per_second);
    
    // Read performance
    let start = Instant::now();
    
    for i in 0..iterations {
        let key = format!("perf_test_{}", i);
        let _value = memory_state_manager.get(StateScope::Global, &key).await?;
    }
    
    let read_duration = start.elapsed();
    let reads_per_second = iterations as f64 / read_duration.as_secs_f64();
    
    println!("      - {} read operations in {:?}", iterations, read_duration);
    println!("      - {:.0} reads per second", reads_per_second);
    
    // Cleanup performance test data
    for i in 0..iterations {
        let key = format!("perf_test_{}", i);
        memory_state_manager.delete(StateScope::Global, &key).await?;
    }
    
    println!("   🧹 Performance test data cleaned up");
    
    // 9. CONCURRENT ACCESS EXAMPLE
    println!("\n9. Concurrent access example...");
    
    let state_manager = Arc::new(memory_state_manager);
    let concurrent_ops = 50;
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for i in 0..concurrent_ops {
        let sm = Arc::clone(&state_manager);
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_{}", i);
            let value = json!({
                "thread_id": i,
                "data": format!("concurrent_data_{}", i)  
            });
            
            sm.set(StateScope::Global, &key, value).await.unwrap();
        });
        handles.push(handle);
    }
    
    // Wait for all concurrent operations
    for handle in handles {
        handle.await?;
    }
    
    let concurrent_duration = start.elapsed();
    let concurrent_ops_per_second = concurrent_ops as f64 / concurrent_duration.as_secs_f64();
    
    println!("   🚀 Concurrent operations:");
    println!("      - {} concurrent writes in {:?}", concurrent_ops, concurrent_duration);
    println!("      - {:.0} concurrent ops per second", concurrent_ops_per_second);
    
    // Verify concurrent data
    for i in 0..5 { // Check first 5
        let key = format!("concurrent_{}", i);
        if let Some(value) = state_manager.get(StateScope::Global, &key).await? {
            println!("      - {}: thread_id = {}", key, value["thread_id"]);
        }
    }
    
    // Cleanup concurrent test data
    for i in 0..concurrent_ops {
        let key = format!("concurrent_{}", i);
        state_manager.delete(StateScope::Global, &key).await?;
    }
    
    println!("\n✅ Basic State Operations Example Completed!");
    println!("\n📋 Summary of demonstrated features:");
    println!("   - StateManager creation with memory and persistent backends");
    println!("   - Basic CRUD operations (Create, Read, Update, Delete)");
    println!("   - Multiple state scopes (Global, Agent, Workflow, Step)");
    println!("   - Error handling patterns");
    println!("   - Performance measurement");
    println!("   - Concurrent access patterns");
    
    println!("\n🔗 Next steps:");
    println!("   - Try agent_persistence.rs for agent state management");
    println!("   - Explore migration_example.rs for schema evolution");
    println!("   - Check backup_creation.rs for data protection");
    println!("   - Review security examples for access control");
    
    // Cleanup happens automatically when temp_dir is dropped
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_operations() {
        let state_manager = StateManager::new().await.unwrap();
        
        // Test set operation
        let test_data = json!({"test": "value"});
        state_manager.set(StateScope::Global, "test_key", test_data.clone()).await.unwrap();
        
        // Test get operation
        let retrieved = state_manager.get(StateScope::Global, "test_key").await.unwrap();
        assert_eq!(retrieved, Some(test_data));
        
        // Test delete operation
        let deleted = state_manager.delete(StateScope::Global, "test_key").await.unwrap();
        assert!(deleted);
        
        // Test get after delete
        let after_delete = state_manager.get(StateScope::Global, "test_key").await.unwrap();
        assert_eq!(after_delete, None);
    }
    
    #[tokio::test]
    async fn test_different_scopes() {
        let state_manager = StateManager::new().await.unwrap();
        
        let test_value = json!("test_data");
        
        // Test different scopes don't interfere
        state_manager.set(StateScope::Global, "same_key", test_value.clone()).await.unwrap();
        state_manager.set(StateScope::Agent("agent1".to_string()), "same_key", json!("different_data")).await.unwrap();
        
        let global_value = state_manager.get(StateScope::Global, "same_key").await.unwrap();
        let agent_value = state_manager.get(StateScope::Agent("agent1".to_string()), "same_key").await.unwrap();
        
        assert_eq!(global_value, Some(test_value));
        assert_eq!(agent_value, Some(json!("different_data")));
    }
}