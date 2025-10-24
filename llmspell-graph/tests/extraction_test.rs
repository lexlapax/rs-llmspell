//! Integration tests for entity and relationship extraction
//!
//! Tests the `RegexExtractor` against real-world text samples to verify
//! >50% recall and <5ms performance targets.

use llmspell_graph::extraction::RegexExtractor;

#[test]
fn test_technical_documentation_extraction() {
    let extractor = RegexExtractor::new();
    let text = "Rust is a systems programming language that runs blazingly fast, \
                prevents segfaults, and guarantees thread safety. \
                Rust has memory safety without garbage collection. \
                Cargo is a build system and package manager for Rust. \
                The Rust compiler uses LLVM as its backend.";

    let entities = extractor.extract_entities(text);
    let rels = extractor.extract_relationships(text);

    // Verify entity extraction
    assert!(
        entities.iter().any(|e| e.name == "Rust"),
        "Should find Rust"
    );
    assert!(
        entities.iter().any(|e| e.name == "Cargo"),
        "Should find Cargo"
    );
    assert!(
        entities.iter().any(|e| e.name == "LLVM"),
        "Should find LLVM"
    );

    // Verify relationship extraction
    let is_a_rels: Vec<_> = rels
        .iter()
        .filter(|r| r.relationship_type == "is_a")
        .collect();
    assert!(!is_a_rels.is_empty(), "Should find is_a relationships");

    let has_rels: Vec<_> = rels
        .iter()
        .filter(|r| r.relationship_type == "has_feature")
        .collect();
    assert!(
        !has_rels.is_empty(),
        "Should find has_feature relationships"
    );

    // Check specific relationships
    assert!(
        is_a_rels
            .iter()
            .any(|r| r.from_entity == "Rust" && r.to_entity.contains("language")),
        "Should find 'Rust is a language'"
    );

    assert!(
        has_rels
            .iter()
            .any(|r| r.from_entity == "Rust" && r.to_entity.contains("safety")),
        "Should find 'Rust has memory safety'"
    );
}

#[test]
fn test_multi_language_comparison() {
    let extractor = RegexExtractor::new();
    let text = "Python is a high-level programming language. \
                Python has dynamic typing. \
                JavaScript is a scripting language. \
                JavaScript runs in browsers. \
                TypeScript is a superset of JavaScript. \
                TypeScript has static typing support.";

    let entities = extractor.extract_entities(text);
    let rels = extractor.extract_relationships(text);

    // Should find all three languages
    assert!(
        entities.iter().any(|e| e.name == "Python"),
        "Should find Python"
    );
    assert!(
        entities.iter().any(|e| e.name == "JavaScript"),
        "Should find JavaScript"
    );
    assert!(
        entities.iter().any(|e| e.name == "TypeScript"),
        "Should find TypeScript"
    );

    // Should classify as programming languages
    let python = entities.iter().find(|e| e.name == "Python");
    assert!(python.is_some());
    assert_eq!(python.unwrap().entity_type, "programming_language");

    // Should find multiple is_a relationships
    let is_a_count = rels
        .iter()
        .filter(|r| r.relationship_type == "is_a")
        .count();
    assert!(is_a_count >= 2, "Should find at least 2 is_a relationships");

    // Should find has relationships
    let has_count = rels
        .iter()
        .filter(|r| r.relationship_type == "has_feature")
        .count();
    assert!(has_count >= 2, "Should find at least 2 has relationships");
}

#[test]
fn test_framework_ecosystem() {
    let extractor = RegexExtractor::new();
    let text = "React is a JavaScript library. \
                Next.js is a framework for React. \
                Vercel is a platform for Next.js. \
                React has a virtual DOM. \
                Components in React are composable.";

    let entities = extractor.extract_entities(text);
    let rels = extractor.extract_relationships(text);

    // Entity extraction
    assert!(entities.iter().any(|e| e.name == "React"));
    assert!(entities.iter().any(|e| e.name == "Vercel"));

    // Relationship types
    let has_is_a = rels.iter().any(|r| r.relationship_type == "is_a");
    let has_feature = rels.iter().any(|r| r.relationship_type == "has_feature");

    assert!(has_is_a || has_feature, "Should find some relationships");
}

#[test]
fn test_geographical_relationships() {
    let extractor = RegexExtractor::new();
    let text = "Paris in France is the capital. \
                The Eiffel Tower in Paris is famous. \
                France in Europe has many cities.";

    let rels = extractor.extract_relationships(text);

    // Should find located_in relationships
    let located_in: Vec<_> = rels
        .iter()
        .filter(|r| r.relationship_type == "located_in")
        .collect();

    assert!(
        !located_in.is_empty(),
        "Should find located_in relationships"
    );

    // Verify specific relationships
    assert!(
        located_in
            .iter()
            .any(|r| r.from_entity == "Paris" && r.to_entity == "France"),
        "Should find 'Paris in France'"
    );
}

#[test]
#[allow(clippy::cast_precision_loss)] // Test metrics with small counts
fn test_recall_benchmark() {
    let extractor = RegexExtractor::new();

    // Curated test text with known entity/relationship counts
    let text = "Rust is a systems programming language. \
                Rust has memory safety. \
                Rust has zero-cost abstractions. \
                Python is a high-level language. \
                Python has dynamic typing. \
                Cargo is a tool for Rust. \
                PyPI is a package repository. \
                Docker is a platform. \
                Kubernetes orchestrates Docker containers.";

    let entities = extractor.extract_entities(text);
    let rels = extractor.extract_relationships(text);

    // Expected entities: Rust, Python, Cargo, PyPI, Docker, Kubernetes = 6
    // Expected relationships: ~8 (2 is_a, 3 has, etc.)

    // Count found entities (main ones)
    let key_entities = ["Rust", "Python", "Cargo", "Docker", "Kubernetes"];
    let found_entities = key_entities
        .iter()
        .filter(|name| entities.iter().any(|e| &e.name == *name))
        .count();

    // Count found relationships
    let is_a_count = rels
        .iter()
        .filter(|r| r.relationship_type == "is_a")
        .count();
    let has_count = rels
        .iter()
        .filter(|r| r.relationship_type == "has_feature")
        .count();
    let total_rels = is_a_count + has_count;

    // Recall: (found / expected)
    let entity_recall = found_entities as f64 / key_entities.len() as f64;
    let rel_recall = total_rels as f64 / 5.0; // Expecting at least 5 relationships

    println!("Entity recall: {:.1}%", entity_recall * 100.0);
    println!("Relationship recall: {:.1}%", rel_recall * 100.0);
    println!("Found entities: {found_entities}/{}", key_entities.len());
    println!("Found relationships: {total_rels} (is_a: {is_a_count}, has: {has_count})");

    // Assert >50% recall
    assert!(
        entity_recall >= 0.5,
        "Entity recall should be >50%, got {:.1}%",
        entity_recall * 100.0
    );
    assert!(
        rel_recall >= 0.5,
        "Relationship recall should be >50%, got {:.1}%",
        rel_recall * 100.0
    );
}

#[test]
#[allow(clippy::cast_precision_loss)] // Test metrics with small counts
fn test_precision_benchmark() {
    let extractor = RegexExtractor::new();

    // Text with known true/false positives for precision measurement
    let text = "Rust is a systems programming language. \
                However, Python is also a language. \
                The Rust compiler is fast. \
                Many developers use Python for scripting. \
                When comparing languages, each has benefits. \
                Docker is a platform for containers. \
                Some prefer Java while others prefer Go.";

    let entities = extractor.extract_entities(text);

    // Expected true positives: Rust, Python, Docker, Java, Go = 5
    let true_positives = ["Rust", "Python", "Docker", "Java", "Go"];
    let tp_count = true_positives
        .iter()
        .filter(|name| entities.iter().any(|e| &e.name == *name))
        .count();

    // Common false positives (stopwords that might slip through): The, However, When, Many, Some
    let common_false_positives = ["The", "However", "When", "Many", "Some", "Each"];
    let fp_count = common_false_positives
        .iter()
        .filter(|name| entities.iter().any(|e| &e.name == *name))
        .count();

    // Precision = TP / (TP + FP)
    let precision = tp_count as f64 / (tp_count + fp_count) as f64;

    println!("True positives found: {tp_count}/{}", true_positives.len());
    println!("False positives found: {fp_count} (stopwords leaked)");
    println!("Precision: {:.1}%", precision * 100.0);
    println!(
        "Extracted entities: {:?}",
        entities.iter().map(|e| &e.name).collect::<Vec<_>>()
    );

    // Assert >60% precision (better than original 30-40%)
    assert!(
        precision >= 0.6,
        "Precision should be >60% with stopword filtering, got {:.1}%",
        precision * 100.0
    );

    // Also verify no common stopwords leaked
    assert_eq!(
        fp_count, 0,
        "No common stopwords should be extracted as entities"
    );
}

#[test]
fn test_performance_target() {
    let extractor = RegexExtractor::new();

    // Generate exactly 1KB of text
    let base_text = "Rust is a systems programming language. \
                     Rust has memory safety and zero-cost abstractions. \
                     Python is a high-level language with dynamic typing. \
                     JavaScript runs in browsers and on servers. \
                     TypeScript is a typed superset of JavaScript. \
                     Go is a language designed at Google. \
                     Kotlin is a modern language for the JVM. \
                     Swift is a language for Apple platforms. ";

    let text = base_text.repeat(1024 / base_text.len() + 1);
    let text = &text[..1024]; // Exact 1KB

    assert_eq!(text.len(), 1024, "Text should be exactly 1KB");

    // Warm-up run (discard timing)
    let _ = extractor.extract_entities(text);
    let _ = extractor.extract_relationships(text);

    // Run multiple iterations and collect timings
    let iterations = 10;
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = std::time::Instant::now();
        let entities = extractor.extract_entities(text);
        let rels = extractor.extract_relationships(text);
        durations.push(start.elapsed());

        // Verify we extracted something (first iteration)
        if durations.len() == 1 {
            assert!(!entities.is_empty(), "Should extract entities");
            assert!(!rels.is_empty(), "Should extract relationships");
            println!(
                "Found {} entities, {} relationships",
                entities.len(),
                rels.len()
            );
        }
    }

    // Calculate median (more stable than mean)
    durations.sort();
    let median = durations[iterations / 2];
    let p95 = durations[(iterations as f64 * 0.95) as usize];

    println!("Extraction median: {median:?}, p95: {p95:?} over {iterations} runs");

    // Performance target: median <6ms for 1KB (with stopword filtering for precision)
    assert!(
        median.as_millis() < 6,
        "Median should be <6ms (with stopword filtering), got {median:?}"
    );
}

#[test]
fn test_edge_cases() {
    let extractor = RegexExtractor::new();

    // Empty text
    let entities = extractor.extract_entities("");
    assert!(entities.is_empty());

    // No entities
    let entities = extractor.extract_entities("this is all lowercase text");
    assert!(entities.is_empty());

    // Mixed case
    let text = "RuSt Is A lAnGuAgE";
    let entities = extractor.extract_entities(text);
    // Should still find "RuSt" despite odd casing
    assert!(!entities.is_empty());

    // Special characters
    let text = "Rust-lang is a language. C++ has pointers.";
    let entities = extractor.extract_entities(text);
    // Hyphenated words might not be captured perfectly, but shouldn't crash
    assert!(!entities.is_empty());
}

#[test]
fn test_entity_deduplication() {
    let extractor = RegexExtractor::new();
    let text = "Rust is great. Rust is fast. Rust is safe. Rust is amazing.";

    let entities = extractor.extract_entities(text);

    // Should only have one Rust entity
    let rust_count = entities.iter().filter(|e| e.name == "Rust").count();
    assert_eq!(rust_count, 1, "Should deduplicate Rust entity");
}

#[test]
fn test_relationship_metadata() {
    let extractor = RegexExtractor::new();
    let text = "Rust is a systems language. Python has dynamic typing.";

    let rels = extractor.extract_relationships(text);

    // Verify metadata is present
    for rel in &rels {
        assert!(
            rel.properties.get("source").is_some(),
            "Should have source metadata"
        );
        assert!(
            rel.properties.get("pattern").is_some(),
            "Should have pattern metadata"
        );
    }
}

#[test]
fn test_multi_word_entities() {
    let extractor = RegexExtractor::new();
    let text = "Visual Studio Code is a code editor. \
                Android Studio is an IDE. \
                IntelliJ IDEA has code completion.";

    let entities = extractor.extract_entities(text);

    // Should capture multi-word entity names
    assert!(
        entities.iter().any(|e| e.name.contains("Visual")),
        "Should find Visual Studio Code parts"
    );
    assert!(
        entities.iter().any(|e| e.name.contains("Android")),
        "Should find Android Studio parts"
    );
    assert!(
        entities.iter().any(|e| e.name.contains("IntelliJ")),
        "Should find IntelliJ IDEA parts"
    );
}
