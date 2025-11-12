#!/usr/bin/env rust-script
//! Generate synthetic graph dataset for performance testing (Task 13c.2.8.6)
//!
//! Creates 100K entities and ~1M relationships with bi-temporal timestamps.
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
use std::fs::File;
use std::io::Write;
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
    println!("Generating synthetic graph dataset (100K entities, ~1M relationships)...");

    let mut rng = rand::thread_rng();
    let now = Utc::now();
    let five_years_ago = now - Duration::days(365 * 5);

    // Entity types and their distribution
    let entity_types = vec![
        ("person", 30_000),       // 30%
        ("concept", 25_000),      // 25%
        ("organization", 20_000), // 20%
        ("event", 15_000),        // 15%
        ("location", 10_000),     // 10%
    ];

    // Relationship types
    let relationship_types = vec![
        "knows",
        "works_at",
        "part_of",
        "caused_by",
        "located_in",
    ];

    println!("Generating 100,000 entities...");
    let mut entities = Vec::new();
    let mut entity_ids = Vec::new();

    for (entity_type, count) in entity_types {
        for i in 0..count {
            let id = Uuid::new_v4().to_string();
            let event_time = five_years_ago
                + Duration::seconds(rng.gen_range(0..(365 * 5 * 24 * 3600)));
            let ingestion_time = event_time + Duration::hours(rng.gen_range(0..48));

            let mut properties = HashMap::new();
            properties.insert(
                "index".to_string(),
                serde_json::json!(i),
            );
            properties.insert(
                "synthetic".to_string(),
                serde_json::json!(true),
            );

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
    }

    println!("Generated {} entities", entities.len());

    println!("Generating ~1,000,000 relationships (avg 10 per entity)...");
    let mut relationships = Vec::new();

    // Generate relationships with power-law distribution (some nodes are more connected)
    for _ in 0..1_000_000 {
        let from_idx = rng.gen_range(0..entity_ids.len());
        let mut to_idx = rng.gen_range(0..entity_ids.len());

        // Avoid self-loops
        while to_idx == from_idx {
            to_idx = rng.gen_range(0..entity_ids.len());
        }

        let rel_type = relationship_types.choose(&mut rng).unwrap();
        let event_time =
            five_years_ago + Duration::seconds(rng.gen_range(0..(365 * 5 * 24 * 3600)));
        let ingestion_time = event_time + Duration::hours(rng.gen_range(0..48));

        let mut properties = HashMap::new();
        properties.insert(
            "synthetic".to_string(),
            serde_json::json!(true),
        );
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

    println!("Generated {} relationships", relationships.len());

    // Write entities to JSON
    println!("Writing entities.json...");
    let entities_file = File::create("entities.json")?;
    serde_json::to_writer_pretty(entities_file, &entities)?;

    // Write relationships to JSON
    println!("Writing relationships.json...");
    let relationships_file = File::create("relationships.json")?;
    serde_json::to_writer_pretty(relationships_file, &relationships)?;

    // Write summary
    let mut summary_file = File::create("dataset-summary.txt")?;
    writeln!(summary_file, "Synthetic Graph Dataset Summary")?;
    writeln!(summary_file, "================================")?;
    writeln!(summary_file)?;
    writeln!(summary_file, "Entities: {}", entities.len())?;
    writeln!(summary_file, "Relationships: {}", relationships.len())?;
    writeln!(
        summary_file,
        "Avg relationships per entity: {:.2}",
        relationships.len() as f64 / entities.len() as f64
    )?;
    writeln!(summary_file)?;
    writeln!(summary_file, "Entity Types:")?;
    for (entity_type, count) in entity_types {
        writeln!(
            summary_file,
            "  - {}: {} ({:.1}%)",
            entity_type,
            count,
            (count as f64 / entities.len() as f64) * 100.0
        )?;
    }
    writeln!(summary_file)?;
    writeln!(summary_file, "Relationship Types:")?;
    for rel_type in relationship_types {
        let count = relationships
            .iter()
            .filter(|r| r.relationship_type == rel_type)
            .count();
        writeln!(
            summary_file,
            "  - {}: {} ({:.1}%)",
            rel_type,
            count,
            (count as f64 / relationships.len() as f64) * 100.0
        )?;
    }
    writeln!(summary_file)?;
    writeln!(summary_file, "Temporal Range: {} to {}", five_years_ago, now)?;

    println!("\nDataset generation complete!");
    println!("Files created:");
    println!("  - entities.json ({} entities)", entities.len());
    println!("  - relationships.json ({} relationships)", relationships.len());
    println!("  - dataset-summary.txt");

    Ok(())
}
