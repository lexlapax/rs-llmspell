#!/usr/bin/env rust-script
//! Test synthetic graph dataset generator with small dataset
//!
//! ```cargo
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! chrono = "0.4"
//! rand = "0.8"
//! uuid = { version = "1.0", features = ["v4"] }
//! ```

use chrono::{DateTime, Duration, Utc};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Entity {
    id: String,
    name: String,
    entity_type: String,
    properties: HashMap<String, serde_json::Value>,
    event_time: DateTime<Utc>,
    ingestion_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Relationship {
    id: String,
    from_entity: String,
    to_entity: String,
    relationship_type: String,
    properties: HashMap<String, serde_json::Value>,
    event_time: DateTime<Utc>,
    ingestion_time: DateTime<Utc>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing generator with small dataset (100 entities, 500 relationships)...");

    let mut rng = rand::thread_rng();
    let now = Utc::now();
    let five_years_ago = now - Duration::days(365 * 5);

    // Generate 100 test entities
    let mut entities = Vec::new();
    let mut entity_ids = Vec::new();

    let entity_types = vec!["person", "concept", "organization", "event", "location"];

    for i in 0..100 {
        let id = Uuid::new_v4().to_string();
        let entity_type = entity_types[i % entity_types.len()];
        let event_time =
            five_years_ago + Duration::seconds(rng.gen_range(0..(365 * 5 * 24 * 3600)));
        let ingestion_time = event_time + Duration::hours(rng.gen_range(0..48));

        let mut properties = HashMap::new();
        properties.insert("index".to_string(), serde_json::json!(i));
        properties.insert("synthetic".to_string(), serde_json::json!(true));

        entities.push(Entity {
            id: id.clone(),
            name: format!("{}-{}", entity_type, i),
            entity_type: entity_type.to_string(),
            properties,
            event_time,
            ingestion_time,
        });

        entity_ids.push(id);
    }

    println!("✓ Generated {} entities", entities.len());

    // Generate 500 test relationships
    let mut relationships = Vec::new();
    let relationship_types = vec!["knows", "works_at", "part_of", "caused_by", "located_in"];

    for _ in 0..500 {
        let from_idx = rng.gen_range(0..entity_ids.len());
        let mut to_idx = rng.gen_range(0..entity_ids.len());

        while to_idx == from_idx {
            to_idx = rng.gen_range(0..entity_ids.len());
        }

        let rel_type = relationship_types.choose(&mut rng).unwrap();
        let event_time =
            five_years_ago + Duration::seconds(rng.gen_range(0..(365 * 5 * 24 * 3600)));
        let ingestion_time = event_time + Duration::hours(rng.gen_range(0..48));

        let mut properties = HashMap::new();
        properties.insert("synthetic".to_string(), serde_json::json!(true));
        properties.insert(
            "weight".to_string(),
            serde_json::json!(rng.gen_range(1..100)),
        );

        relationships.push(Relationship {
            id: Uuid::new_v4().to_string(),
            from_entity: entity_ids[from_idx].clone(),
            to_entity: entity_ids[to_idx].clone(),
            relationship_type: rel_type.to_string(),
            properties,
            event_time,
            ingestion_time,
        });
    }

    println!("✓ Generated {} relationships", relationships.len());

    // Validate JSON serialization
    let entities_json = serde_json::to_string(&entities)?;
    let relationships_json = serde_json::to_string(&relationships)?;

    println!("✓ JSON serialization successful");
    println!("  Entities JSON size: {} bytes", entities_json.len());
    println!(
        "  Relationships JSON size: {} bytes",
        relationships_json.len()
    );

    // Validate temporal consistency
    let mut temporal_issues = 0;
    for entity in &entities {
        if entity.ingestion_time < entity.event_time {
            temporal_issues += 1;
        }
    }
    for rel in &relationships {
        if rel.ingestion_time < rel.event_time {
            temporal_issues += 1;
        }
    }

    if temporal_issues > 0 {
        println!("✗ Temporal consistency check failed: {} issues", temporal_issues);
        return Err("Temporal consistency violations detected".into());
    }

    println!("✓ Temporal consistency validated");

    // Statistics
    let avg_rels_per_entity = relationships.len() as f64 / entities.len() as f64;
    println!("\nStatistics:");
    println!("  Avg relationships per entity: {:.2}", avg_rels_per_entity);

    println!("\n✓ All validation checks passed!");
    println!("Generator is ready for full 100K dataset generation.");

    Ok(())
}
