//! Baseline measurement for DMR and NDCG@10 metrics
//!
//! Establishes production-scale performance baselines:
//! - **DMR (Decision Match Rate)**: Consolidation accuracy (target: >90%)
//! - **NDCG@10**: Retrieval quality with BM25-only reranking (target: >0.65)
//!
//! # Dataset
//!
//! - **Current**: 50 episodic records (5 conversations × 10 records)
//! - **Production-scale**: Expandable to 500 records (50 conversations × 10 records)
//!   by duplicating conversation patterns with variations
//!
//! # Requirements
//!
//! - Running Ollama instance with llama3.2:3b model
//! - Set `OLLAMA_HOST` environment variable (default: <http://localhost:11434>)
//! - Test skips gracefully if Ollama is unavailable
//!
//! # Performance
//!
//! - **Current**: ~40-60s (50 records × ~0.8s/record)
//! - **Production-scale**: ~7-10min (500 records × ~0.8s/record)

mod e2e;

use llmspell_context::retrieval::BM25Retriever;
use llmspell_memory::consolidation::ConsolidationEngine;
use llmspell_memory::episodic::InMemoryEpisodicMemory;
use llmspell_memory::traits::EpisodicMemory as EpisodicMemoryTrait;
use llmspell_memory::types::EpisodicEntry;
use serial_test::serial;
use std::collections::HashMap;
use tracing::{debug, error, info};

use e2e::helpers::{create_test_engine, GroundTruthDecision};

/// Test dataset: 5 conversations × 10 records = 50 total
///
/// ## Conversations:
///
/// 1. **Rust Programming Q&A** (10 records) - Discussion about Rust features
/// 2. **Python Debugging** (10 records) - Troubleshooting import errors
/// 3. **General Knowledge** (10 records) - Facts about programming languages
/// 4. **Casual Chat** (10 records) - Informal conversation
/// 5. **Technical Explanation** (10 records) - Deep dive into memory management
///
/// ## Expandability to 500 records:
///
/// To expand to production-scale (500 records):
/// 1. Duplicate each conversation pattern 10x with variations:
///    - Change programming language (Rust → Go, Python → Ruby, etc.)
///    - Change topic (memory safety → type safety, debugging → testing, etc.)
///    - Vary phrasing while keeping semantic similarity
/// 2. Add conversation categories:
///    - API design discussions
///    - Algorithm optimization
///    - Database queries
///    - System architecture
///    - Code review feedback
#[derive(Debug, Clone)]
struct ConversationDataset {
    /// Conversation ID
    id: String,
    /// Episodic records for this conversation
    records: Vec<EpisodicEntry>,
    /// Ground truth: expected entities to be added by LLM consolidation
    expected_entities: Vec<GroundTruthDecision>,
}

/// Create test dataset with 50 episodic records
#[allow(clippy::too_many_lines)]
fn create_test_dataset() -> Vec<ConversationDataset> {
    vec![
        // Conversation 1: Rust Programming Q&A (10 records)
        ConversationDataset {
            id: "rust-qa".to_string(),
            records: vec![
                EpisodicEntry::new(
                    "rust-qa".to_string(),
                    "user".to_string(),
                    "What is Rust?".to_string(),
                ),
                EpisodicEntry::new(
                    "rust-qa".to_string(),
                    "assistant".to_string(),
                    "Rust is a systems programming language focused on safety and performance.".to_string(),
                ),
                EpisodicEntry::new(
                    "rust-qa".to_string(),
                    "user".to_string(),
                    "What are Rust's key features?".to_string(),
                ),
                EpisodicEntry::new(
                    "rust-qa".to_string(),
                    "assistant".to_string(),
                    "Rust has memory safety without garbage collection via ownership.".to_string(),
                ),
                EpisodicEntry::new(
                    "rust-qa".to_string(),
                    "user".to_string(),
                    "Tell me about zero-cost abstractions.".to_string(),
                ),
                EpisodicEntry::new(
                    "rust-qa".to_string(),
                    "assistant".to_string(),
                    "Rust provides zero-cost abstractions - abstractions that compile to efficient machine code.".to_string(),
                ),
                EpisodicEntry::new(
                    "rust-qa".to_string(),
                    "user".to_string(),
                    "How does Rust handle concurrency?".to_string(),
                ),
                EpisodicEntry::new(
                    "rust-qa".to_string(),
                    "assistant".to_string(),
                    "Rust prevents data races at compile time using ownership and type system.".to_string(),
                ),
                EpisodicEntry::new(
                    "rust-qa".to_string(),
                    "user".to_string(),
                    "What about async in Rust?".to_string(),
                ),
                EpisodicEntry::new(
                    "rust-qa".to_string(),
                    "assistant".to_string(),
                    "Rust has async/await for asynchronous programming with zero-cost futures.".to_string(),
                ),
            ],
            expected_entities: vec![
                GroundTruthDecision::Add {
                    entity_id: "rust".to_string(),
                },
                GroundTruthDecision::Add {
                    entity_id: "memory-safety".to_string(),
                },
                GroundTruthDecision::Add {
                    entity_id: "zero-cost-abstractions".to_string(),
                },
            ],
        },
        // Conversation 2: Python Debugging (10 records)
        ConversationDataset {
            id: "python-debug".to_string(),
            records: vec![
                EpisodicEntry::new(
                    "python-debug".to_string(),
                    "user".to_string(),
                    "I'm getting ImportError in Python.".to_string(),
                ),
                EpisodicEntry::new(
                    "python-debug".to_string(),
                    "assistant".to_string(),
                    "ImportError usually means the module isn't installed or isn't in PYTHONPATH.".to_string(),
                ),
                EpisodicEntry::new(
                    "python-debug".to_string(),
                    "user".to_string(),
                    "How do I check installed packages?".to_string(),
                ),
                EpisodicEntry::new(
                    "python-debug".to_string(),
                    "assistant".to_string(),
                    "Use 'pip list' to see installed packages.".to_string(),
                ),
                EpisodicEntry::new(
                    "python-debug".to_string(),
                    "user".to_string(),
                    "The package is installed but still getting ImportError.".to_string(),
                ),
                EpisodicEntry::new(
                    "python-debug".to_string(),
                    "assistant".to_string(),
                    "Check if you're using the correct Python environment. Virtual environments can cause this.".to_string(),
                ),
                EpisodicEntry::new(
                    "python-debug".to_string(),
                    "user".to_string(),
                    "How do I activate a virtual environment?".to_string(),
                ),
                EpisodicEntry::new(
                    "python-debug".to_string(),
                    "assistant".to_string(),
                    "Run 'source venv/bin/activate' on Linux/Mac or 'venv\\Scripts\\activate' on Windows.".to_string(),
                ),
                EpisodicEntry::new(
                    "python-debug".to_string(),
                    "user".to_string(),
                    "That fixed it! Thanks.".to_string(),
                ),
                EpisodicEntry::new(
                    "python-debug".to_string(),
                    "assistant".to_string(),
                    "Great! Virtual environment isolation prevents many dependency conflicts.".to_string(),
                ),
            ],
            expected_entities: vec![
                GroundTruthDecision::Add {
                    entity_id: "python".to_string(),
                },
                GroundTruthDecision::Add {
                    entity_id: "importerror".to_string(),
                },
                GroundTruthDecision::Add {
                    entity_id: "virtual-environment".to_string(),
                },
            ],
        },
        // Conversation 3: General Knowledge (10 records)
        ConversationDataset {
            id: "general-knowledge".to_string(),
            records: vec![
                EpisodicEntry::new(
                    "general-knowledge".to_string(),
                    "user".to_string(),
                    "What are the most popular programming languages?".to_string(),
                ),
                EpisodicEntry::new(
                    "general-knowledge".to_string(),
                    "assistant".to_string(),
                    "Python, JavaScript, Java, C++, and Go are among the most popular.".to_string(),
                ),
                EpisodicEntry::new(
                    "general-knowledge".to_string(),
                    "user".to_string(),
                    "Which one is best for beginners?".to_string(),
                ),
                EpisodicEntry::new(
                    "general-knowledge".to_string(),
                    "assistant".to_string(),
                    "Python is often recommended for beginners due to its simple syntax.".to_string(),
                ),
                EpisodicEntry::new(
                    "general-knowledge".to_string(),
                    "user".to_string(),
                    "What about for web development?".to_string(),
                ),
                EpisodicEntry::new(
                    "general-knowledge".to_string(),
                    "assistant".to_string(),
                    "JavaScript is essential for web development, both frontend and backend.".to_string(),
                ),
                EpisodicEntry::new(
                    "general-knowledge".to_string(),
                    "user".to_string(),
                    "Is TypeScript better than JavaScript?".to_string(),
                ),
                EpisodicEntry::new(
                    "general-knowledge".to_string(),
                    "assistant".to_string(),
                    "TypeScript adds static typing to JavaScript, improving code reliability.".to_string(),
                ),
                EpisodicEntry::new(
                    "general-knowledge".to_string(),
                    "user".to_string(),
                    "What's the difference between compiled and interpreted languages?".to_string(),
                ),
                EpisodicEntry::new(
                    "general-knowledge".to_string(),
                    "assistant".to_string(),
                    "Compiled languages translate to machine code before execution. Interpreted languages execute line-by-line.".to_string(),
                ),
            ],
            expected_entities: vec![
                GroundTruthDecision::Add {
                    entity_id: "python".to_string(),
                },
                GroundTruthDecision::Add {
                    entity_id: "javascript".to_string(),
                },
                GroundTruthDecision::Add {
                    entity_id: "typescript".to_string(),
                },
            ],
        },
        // Conversation 4: Casual Chat (10 records)
        ConversationDataset {
            id: "casual-chat".to_string(),
            records: vec![
                EpisodicEntry::new(
                    "casual-chat".to_string(),
                    "user".to_string(),
                    "Hey, how's it going?".to_string(),
                ),
                EpisodicEntry::new(
                    "casual-chat".to_string(),
                    "assistant".to_string(),
                    "Hello! I'm doing well, thanks for asking.".to_string(),
                ),
                EpisodicEntry::new(
                    "casual-chat".to_string(),
                    "user".to_string(),
                    "What are you working on today?".to_string(),
                ),
                EpisodicEntry::new(
                    "casual-chat".to_string(),
                    "assistant".to_string(),
                    "I'm helping with programming questions and technical discussions.".to_string(),
                ),
                EpisodicEntry::new(
                    "casual-chat".to_string(),
                    "user".to_string(),
                    "That sounds interesting!".to_string(),
                ),
                EpisodicEntry::new(
                    "casual-chat".to_string(),
                    "assistant".to_string(),
                    "It is! I enjoy helping people learn and solve problems.".to_string(),
                ),
                EpisodicEntry::new(
                    "casual-chat".to_string(),
                    "user".to_string(),
                    "Do you have any favorite programming languages?".to_string(),
                ),
                EpisodicEntry::new(
                    "casual-chat".to_string(),
                    "assistant".to_string(),
                    "I appreciate Rust for its safety and Python for its simplicity.".to_string(),
                ),
                EpisodicEntry::new(
                    "casual-chat".to_string(),
                    "user".to_string(),
                    "Good choices!".to_string(),
                ),
                EpisodicEntry::new(
                    "casual-chat".to_string(),
                    "assistant".to_string(),
                    "Thanks! Each language has its strengths for different use cases.".to_string(),
                ),
            ],
            expected_entities: vec![
                // Casual chat may not generate strong entities (NOOP decisions expected)
                // But LLM might extract "programming" or "rust"/"python" mentions
            ],
        },
        // Conversation 5: Technical Explanation (10 records)
        ConversationDataset {
            id: "memory-management".to_string(),
            records: vec![
                EpisodicEntry::new(
                    "memory-management".to_string(),
                    "user".to_string(),
                    "Explain memory management in programming.".to_string(),
                ),
                EpisodicEntry::new(
                    "memory-management".to_string(),
                    "assistant".to_string(),
                    "Memory management involves allocating and freeing memory for program data.".to_string(),
                ),
                EpisodicEntry::new(
                    "memory-management".to_string(),
                    "user".to_string(),
                    "What's garbage collection?".to_string(),
                ),
                EpisodicEntry::new(
                    "memory-management".to_string(),
                    "assistant".to_string(),
                    "Garbage collection automatically reclaims memory no longer in use.".to_string(),
                ),
                EpisodicEntry::new(
                    "memory-management".to_string(),
                    "user".to_string(),
                    "Which languages use garbage collection?".to_string(),
                ),
                EpisodicEntry::new(
                    "memory-management".to_string(),
                    "assistant".to_string(),
                    "Java, Python, JavaScript, and Go all use garbage collection.".to_string(),
                ),
                EpisodicEntry::new(
                    "memory-management".to_string(),
                    "user".to_string(),
                    "What about manual memory management?".to_string(),
                ),
                EpisodicEntry::new(
                    "memory-management".to_string(),
                    "assistant".to_string(),
                    "C and C++ require manual memory management with malloc/free and new/delete.".to_string(),
                ),
                EpisodicEntry::new(
                    "memory-management".to_string(),
                    "user".to_string(),
                    "What's the advantage of Rust's approach?".to_string(),
                ),
                EpisodicEntry::new(
                    "memory-management".to_string(),
                    "assistant".to_string(),
                    "Rust uses ownership for memory safety without garbage collection overhead.".to_string(),
                ),
            ],
            expected_entities: vec![
                GroundTruthDecision::Add {
                    entity_id: "memory-management".to_string(),
                },
                GroundTruthDecision::Add {
                    entity_id: "garbage-collection".to_string(),
                },
                GroundTruthDecision::Add {
                    entity_id: "rust".to_string(),
                },
            ],
        },
    ]
}

/// Retrieval queries for NDCG@10 measurement
///
/// Each query has relevance labels: which conversation IDs contain relevant information
#[derive(Debug, Clone)]
struct RetrievalQuery {
    /// Query text
    query: String,
    /// Relevant conversation IDs (for NDCG calculation)
    relevant_conversations: Vec<String>,
}

/// Create retrieval test queries with relevance labels
fn create_retrieval_queries() -> Vec<RetrievalQuery> {
    vec![
        RetrievalQuery {
            query: "What are Rust's memory safety features?".to_string(),
            relevant_conversations: vec!["rust-qa".to_string(), "memory-management".to_string()],
        },
        RetrievalQuery {
            query: "How to fix Python ImportError?".to_string(),
            relevant_conversations: vec!["python-debug".to_string()],
        },
        RetrievalQuery {
            query: "Best programming language for beginners".to_string(),
            relevant_conversations: vec!["general-knowledge".to_string()],
        },
        RetrievalQuery {
            query: "Explain garbage collection".to_string(),
            relevant_conversations: vec!["memory-management".to_string()],
        },
        RetrievalQuery {
            query: "JavaScript vs TypeScript".to_string(),
            relevant_conversations: vec!["general-knowledge".to_string()],
        },
    ]
}

/// Calculate NDCG@10 for a single query
///
/// NDCG (Normalized Discounted Cumulative Gain) measures ranking quality:
/// - 1.0 = perfect ranking (all relevant items at top)
/// - 0.0 = worst ranking (no relevant items retrieved)
///
/// Formula: `DCG@k` = Σ (`2^rel_i` - 1) / log2(i + 1)
/// `NDCG@k` = `DCG@k` / `IDCG@k` (normalized by ideal DCG)
#[allow(clippy::cast_precision_loss)]
fn calculate_ndcg_at_10(
    retrieved_session_ids: &[String],
    relevant_conversations: &[String],
) -> f64 {
    if relevant_conversations.is_empty() {
        return 1.0; // No relevant docs = perfect score (vacuous truth)
    }

    // Calculate DCG@10
    let mut dcg = 0.0;
    for (i, session_id) in retrieved_session_ids.iter().take(10).enumerate() {
        let relevance: f64 = if relevant_conversations.contains(session_id) {
            1.0 // Binary relevance: 1 if relevant, 0 otherwise
        } else {
            0.0
        };

        let rank = (i + 2) as f64; // Rank starts at 1, but log2(1) = 0, so we use i+2
        dcg += (relevance.exp2() - 1.0) / rank.log2();
    }

    // Calculate IDCG@10 (ideal DCG - all relevant docs at top)
    let mut idcg = 0.0;
    let ideal_relevance_count = relevant_conversations.len().min(10);
    for i in 0..ideal_relevance_count {
        let rank = (i + 2) as f64;
        idcg += 1.0 / rank.log2(); // 2^1 - 1 = 1, so simplified
    }

    if idcg == 0.0 {
        return 0.0;
    }

    dcg / idcg
}

/// Baseline measurement test: DMR and NDCG@10
///
/// Measures:
/// - **DMR**: Decision Match Rate for LLM consolidation accuracy
/// - **NDCG@10**: Retrieval quality for BM25-based context retrieval
///
/// # Test Flow
///
/// 1. Load 50 episodic records (5 conversations)
/// 2. Run LLM consolidation with real Ollama
/// 3. Calculate DMR by comparing LLM decisions to ground truth
/// 4. Run BM25 retrieval on test queries
/// 5. Calculate NDCG@10 by comparing rankings to relevance labels
/// 6. Log baseline metrics
#[allow(clippy::cast_precision_loss)]
#[ignore = "Requires Ollama - run individually for baseline measurement"]
#[tokio::test]
#[serial]
async fn test_baseline_dmr_and_ndcg10() {
    // Initialize tracing
    let _ = tracing_subscriber::fmt()
        .with_env_filter("llmspell_memory=info,llmspell_context=info")
        .with_test_writer()
        .try_init();

    info!("=== BASELINE MEASUREMENT: DMR and NDCG@10 ===");

    // Check Ollama availability
    if !crate::e2e::check_ollama_available().await {
        eprintln!("Skipping baseline measurement - Ollama unavailable");
        return;
    }

    info!("✓ Ollama available - starting baseline measurement");

    // ============================================================================
    // STEP 1: Load test dataset (50 records, 5 conversations)
    // ============================================================================
    info!("STEP 1: Loading test dataset (50 episodic records)");

    let dataset = create_test_dataset();
    let total_records: usize = dataset.iter().map(|c| c.records.len()).sum();

    info!(
        "Dataset loaded: {} conversations, {} records",
        dataset.len(),
        total_records
    );

    // ============================================================================
    // STEP 2: Setup test infrastructure
    // ============================================================================
    info!("STEP 2: Setting up test engine and episodic memory");

    let test_engine = create_test_engine().await;
    let episodic_memory = InMemoryEpisodicMemory::default();

    // Add all records to episodic memory
    for conversation in &dataset {
        for entry in &conversation.records {
            episodic_memory
                .add(entry.clone())
                .await
                .expect("Failed to add entry to episodic memory");
        }
    }

    debug!("Added {} records to episodic memory", total_records);

    // ============================================================================
    // STEP 3: Run LLM consolidation and measure DMR
    // ============================================================================
    info!("STEP 3: Running LLM consolidation (this may take 40-60s)");

    let mut per_conversation_dmr: HashMap<String, f64> = HashMap::new();

    for conversation in &dataset {
        info!(
            "  Consolidating conversation: {} ({} records)",
            conversation.id,
            conversation.records.len()
        );

        // Small delay to avoid overwhelming Ollama
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        let mut entries_for_consolidation = conversation.records.clone();
        let result = test_engine
            .llm_engine
            .consolidate(&[&conversation.id], &mut entries_for_consolidation)
            .await;

        match result {
            Ok(consolidation_result) => {
                debug!(
                    "    Processed: {}, Added: {}, Updated: {}, Failed: {}",
                    consolidation_result.entries_processed,
                    consolidation_result.entities_added,
                    consolidation_result.entities_updated,
                    consolidation_result.entries_failed
                );

                // For DMR calculation, we need actual decisions
                // ConsolidationResult only has counts, so we use type-level validation:
                // - If entities_added > 0, at least one ADD decision was made
                // - If entities_updated > 0, at least one UPDATE decision was made
                // This is a simplified DMR measurement (type-level only)

                // For production-scale DMR, we would need the engine to expose
                // ConsolidationResponse with actual DecisionPayload instances

                // Calculate per-conversation DMR (type-level approximation)
                let expected_adds = conversation
                    .expected_entities
                    .iter()
                    .filter(|d| matches!(d, GroundTruthDecision::Add { .. }))
                    .count();

                let type_level_dmr = if expected_adds > 0 {
                    if consolidation_result.entities_added > 0 {
                        1.0 // At least one ADD decision made (type matches)
                    } else {
                        0.0
                    }
                } else {
                    1.0 // No ADDs expected, so NOOP is correct
                };

                per_conversation_dmr.insert(conversation.id.clone(), type_level_dmr);
            }
            Err(e) => {
                error!("    Consolidation failed: {}", e);
                per_conversation_dmr.insert(conversation.id.clone(), 0.0);
            }
        }
    }

    // Calculate overall DMR (type-level approximation)
    let overall_dmr: f64 =
        per_conversation_dmr.values().sum::<f64>() / per_conversation_dmr.len() as f64;

    info!("✓ Consolidation complete");
    info!("  Overall DMR (type-level): {:.1}%", overall_dmr * 100.0);
    for (conv_id, dmr) in &per_conversation_dmr {
        debug!("    {}: {:.1}%", conv_id, dmr * 100.0);
    }

    // ============================================================================
    // STEP 4: Run BM25 retrieval and measure NDCG@10
    // ============================================================================
    info!("STEP 4: Running BM25 retrieval for NDCG@10 measurement");

    let retrieval_queries = create_retrieval_queries();
    let bm25_retriever = BM25Retriever::new();

    let mut per_query_ndcg: HashMap<String, f64> = HashMap::new();

    for query_data in &retrieval_queries {
        debug!("  Query: {}", query_data.query);

        let retrieved_chunks = bm25_retriever
            .retrieve_from_memory(&query_data.query, &episodic_memory, 20, 10)
            .await
            .expect("BM25 retrieval should succeed");

        // Extract session IDs from retrieved chunks
        // Note: Deduplicate session IDs since BM25 returns chunks, not conversations
        // Multiple chunks from same conversation should count as one retrieval
        //
        // Use seen set to track unique conversations in order
        let mut retrieved_session_ids = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for chunk in &retrieved_chunks {
            if seen.insert(chunk.source.clone()) {
                retrieved_session_ids.push(chunk.source.clone());
            }
        }

        // Calculate NDCG@10
        let ndcg = calculate_ndcg_at_10(&retrieved_session_ids, &query_data.relevant_conversations);

        per_query_ndcg.insert(query_data.query.clone(), ndcg);
        debug!("    NDCG@10: {:.3}", ndcg);
    }

    let mean_ndcg10: f64 = per_query_ndcg.values().sum::<f64>() / per_query_ndcg.len() as f64;

    info!("✓ Retrieval complete");
    info!("  Mean NDCG@10: {:.3}", mean_ndcg10);

    // ============================================================================
    // STEP 5: Log baseline results
    // ============================================================================
    info!("=== BASELINE RESULTS ===");
    info!(
        "Dataset: {} conversations, {} records",
        dataset.len(),
        total_records
    );
    info!("DMR (Decision Match Rate): {:.1}%", overall_dmr * 100.0);
    info!("NDCG@10 (BM25-only): {:.3}", mean_ndcg10);
    info!("========================");

    let dmr_percent = overall_dmr * 100.0;
    eprintln!("\n✓ Baseline measurement complete");
    eprintln!("  DMR: {dmr_percent:.1}%");
    eprintln!("  NDCG@10: {mean_ndcg10:.3}");
    eprintln!("  Dataset: {total_records} records");

    // Assertions for baseline targets
    assert!(
        overall_dmr >= 0.70,
        "DMR should be ≥70% (type-level validation)"
    );
    assert!(
        mean_ndcg10 >= 0.50,
        "NDCG@10 should be ≥0.50 with BM25-only (target: 0.65)"
    );
}
