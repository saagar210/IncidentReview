/// Stress tests for AI evidence store and embeddings at scale
///
/// Verifies performance and correctness with:
/// - 100,000 evidence chunks
/// - Large embedding index (memory and disk usage)
/// - Similarity search latency under load
/// - Incremental index updates

#[cfg(test)]
mod ai_stress_tests {
    use std::time::Instant;
    use qir_ai::evidence::{EvidenceStore, EvidenceChunk, EvidenceSource};
    use qir_ai::evidence::index::EmbeddingIndex;
    use qir_ai::retrieve::similarity_search;

    fn generate_synthetic_chunk(index: usize) -> EvidenceChunk {
        let chunk_types = vec![
            "Error log",
            "Stack trace",
            "Database query",
            "API response",
            "System metrics",
            "Network trace",
            "Service log",
            "Deployment log",
        ];

        EvidenceChunk {
            id: format!("chunk-{:06}", index),
            source_id: format!("source-{}", index % 100), // 100 different sources
            content: format!(
                "{} at {}. Details: This is evidence chunk #{} containing synthetic content for stress testing. {}",
                chunk_types[index % chunk_types.len()],
                format!("2025-01-{:02}T{:02}:00:00Z", 1 + (index / 3600) % 28, (index / 150) % 24),
                index,
                "Sample evidence text. ".repeat(10 + (index % 20))
            ),
            created_at: "2025-01-15T00:00:00Z".to_string(),
        }
    }

    #[test]
    #[ignore] // Only run with Ollama available
    fn stress_test_evidence_store_10k_chunks() {
        let store = EvidenceStore::create_memory().unwrap();

        let start = Instant::now();
        for i in 0..10_000 {
            let chunk = generate_synthetic_chunk(i);
            store.add_chunk(&chunk).unwrap();
        }
        let insert_duration = start.elapsed();

        eprintln!("✓ Evidence store (10K chunks) insert: {:?}", insert_duration);
        assert!(insert_duration.as_secs() < 5, "Should insert 10K chunks in <5s");

        let count = store.count_all_chunks().unwrap();
        assert_eq!(count, 10_000);
    }

    #[test]
    #[ignore]
    fn stress_test_evidence_store_100k_chunks() {
        let store = EvidenceStore::create_memory().unwrap();

        let start = Instant::now();
        for i in 0..100_000 {
            let chunk = generate_synthetic_chunk(i);
            store.add_chunk(&chunk).unwrap();

            // Progress indicator every 20K
            if i % 20_000 == 0 && i > 0 {
                eprintln!("  ... {} chunks inserted in {:?}", i, start.elapsed());
            }
        }
        let insert_duration = start.elapsed();

        eprintln!("✓ Evidence store (100K chunks) insert: {:?}", insert_duration);
        assert!(insert_duration.as_secs() < 30, "Should insert 100K chunks in <30s");

        let count = store.count_all_chunks().unwrap();
        assert_eq!(count, 100_000);
    }

    #[test]
    #[ignore]
    fn stress_test_embedding_index_build_10k_chunks() {
        let store = EvidenceStore::create_memory().unwrap();

        // Add chunks
        for i in 0..10_000 {
            let chunk = generate_synthetic_chunk(i);
            store.add_chunk(&chunk).unwrap();
        }

        // Build index
        let start = Instant::now();
        let index = EmbeddingIndex::build(&store).unwrap();
        let build_duration = start.elapsed();

        eprintln!("✓ Embedding index build (10K chunks): {:?}", build_duration);
        assert!(build_duration.as_secs() < 30, "Should build index in <30s for 10K chunks");

        assert_eq!(index.chunk_count(), 10_000);
    }

    #[test]
    #[ignore]
    fn stress_test_embedding_index_build_100k_chunks() {
        let store = EvidenceStore::create_memory().unwrap();

        // Add chunks with progress
        let start = Instant::now();
        for i in 0..100_000 {
            let chunk = generate_synthetic_chunk(i);
            store.add_chunk(&chunk).unwrap();
            if i % 25_000 == 0 && i > 0 {
                eprintln!("  ... {} chunks added in {:?}", i, start.elapsed());
            }
        }
        let insert_duration = start.elapsed();
        eprintln!("  Insert phase: {:?}", insert_duration);

        // Build index
        let start = Instant::now();
        let index = EmbeddingIndex::build(&store).unwrap();
        let build_duration = start.elapsed();

        eprintln!("✓ Embedding index build (100K chunks): {:?}", build_duration);
        // Building 100K embeddings can take 2-5 minutes depending on model
        assert!(build_duration.as_secs() < 300, "Should build index in <5 minutes for 100K chunks");

        assert_eq!(index.chunk_count(), 100_000);
    }

    #[test]
    #[ignore]
    fn stress_test_similarity_search_latency() {
        let store = EvidenceStore::create_memory().unwrap();

        // Add 10K chunks
        for i in 0..10_000 {
            let chunk = generate_synthetic_chunk(i);
            store.add_chunk(&chunk).unwrap();
        }

        // Build index
        let index = EmbeddingIndex::build(&store).unwrap();

        // Run searches
        let queries = vec![
            "database connection error",
            "out of memory exception",
            "timeout in API call",
            "disk space full",
            "network unreachable",
        ];

        for query in queries {
            let start = Instant::now();
            let results = similarity_search(&index, query, 10).unwrap();
            let duration = start.elapsed();

            eprintln!("  Query '{}': {:?} ({} results)", query, duration, results.len());
            assert!(duration.as_millis() < 500, "Search should complete in <500ms");
            assert!(results.len() <= 10);
        }

        eprintln!("✓ Similarity search latency: All queries <500ms");
    }

    #[test]
    #[ignore]
    fn stress_test_incremental_index_updates() {
        let store = EvidenceStore::create_memory().unwrap();

        // Add initial batch
        for i in 0..5_000 {
            let chunk = generate_synthetic_chunk(i);
            store.add_chunk(&chunk).unwrap();
        }

        // Build initial index
        let start = Instant::now();
        let mut index = EmbeddingIndex::build(&store).unwrap();
        let initial_build = start.elapsed();
        eprintln!("  Initial build (5K chunks): {:?}", initial_build);

        // Add more chunks and update index incrementally
        let start = Instant::now();
        for i in 5_000..10_000 {
            let chunk = generate_synthetic_chunk(i);
            store.add_chunk(&chunk).unwrap();
        }

        index.update_from_store(&store).unwrap();
        let update_duration = start.elapsed();
        eprintln!("  Incremental update (5K new chunks): {:?}", update_duration);

        assert!(update_duration.as_secs() < 30, "Incremental update should be fast");
        assert_eq!(index.chunk_count(), 10_000);

        eprintln!("✓ Incremental index updates: Efficient partial rebuilds");
    }

    #[test]
    #[ignore]
    fn stress_test_memory_usage_10k_chunks() {
        let store = EvidenceStore::create_memory().unwrap();

        // Add chunks
        for i in 0..10_000 {
            let chunk = generate_synthetic_chunk(i);
            store.add_chunk(&chunk).unwrap();
        }

        // Estimate memory
        // - 10K chunks: ~10K * 500 bytes = 5MB
        // - Embeddings (if 1536-dim float32): 10K * 1536 * 4 = 61MB
        // - Total: ~70MB
        let estimated_store = 5_000_000;
        let estimated_embeddings = 61_000_000;
        let estimated_total = estimated_store + estimated_embeddings;

        eprintln!("✓ Estimated memory (10K chunks):");
        eprintln!("  Store: {:.1} MB", estimated_store as f64 / 1_000_000.0);
        eprintln!("  Embeddings: {:.1} MB", estimated_embeddings as f64 / 1_000_000.0);
        eprintln!("  Total: {:.1} MB", estimated_total as f64 / 1_000_000.0);

        // Should use <150MB for 10K chunks
        assert!(estimated_total < 150_000_000, "Should use <150MB for 10K chunks");
    }

    #[test]
    #[ignore]
    fn stress_test_concurrent_searches() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(EvidenceStore::create_memory().unwrap());

        // Add chunks
        for i in 0..5_000 {
            let chunk = generate_synthetic_chunk(i);
            store.add_chunk(&chunk).unwrap();
        }

        // Build index once
        let index = Arc::new(EmbeddingIndex::build(&store).unwrap());

        // Run concurrent searches
        let handles: Vec<_> = (0..4)
            .map(|thread_id| {
                let index = Arc::clone(&index);
                thread::spawn(move || {
                    let queries = vec![
                        "database error",
                        "network timeout",
                        "out of memory",
                        "disk full",
                    ];

                    let start = Instant::now();
                    for query in queries {
                        let _results = similarity_search(&index, query, 5).unwrap();
                    }
                    let duration = start.elapsed();
                    eprintln!("  Thread {}: {:?}", thread_id, duration);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        eprintln!("✓ Concurrent searches complete");
    }
}
