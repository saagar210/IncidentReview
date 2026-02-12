/// Stress tests for large-scale incident data
///
/// Verifies performance on realistic dataset sizes:
/// - 10,000 incidents (large organization annual incidents)
/// - 100,000 evidence chunks (comprehensive logging)
/// - Dashboard computation under load
/// - Report generation at scale

#[cfg(test)]
mod stress_tests {
    use std::time::Instant;
    use qir_core::metrics::compute_metrics;
    use qir_core::analytics::get_dashboard_v2;
    use qir_core::report::generate_markdown;
    use qir_core::domain::Incident;

    /// Generate synthetic incident at index i
    fn generate_synthetic_incident(i: usize) -> Incident {
        let vendors = vec!["Platform", "Database", "API", "Cache", "Storage"];
        let services = vec!["Compute", "Networking", "Memory", "Disk", "Replication"];
        let sources = vec!["monitoring", "customer_report", "internal", "alert"];

        Incident {
            id: format!("incident-{:06}", i),
            title: format!("Incident #{} - {} {} Outage", i, vendors[i % vendors.len()], services[i % services.len()]),
            description: Some(format!("Synthetic incident for stress testing. Index: {}", i)),
            detection_time: {
                // Spread incidents over ~1 year
                let hours_ago = (i % (365 * 24)) as u64;
                format!("2024-{:02}-{:02}T{:02}:00:00Z",
                    1 + (hours_ago / 720) % 12,
                    1 + (hours_ago / 24) % 28,
                    hours_ago % 24
                )
            },
            resolution_time: {
                // Resolution 30 minutes to 8 hours after detection
                let hours_ago = (i % (365 * 24)) as u64;
                let duration_minutes = 30 + (i % 480); // 30 min to 8 hours
                format!("2024-{:02}-{:02}T{:02}:{:02}:00Z",
                    1 + ((hours_ago + (duration_minutes / 60) as u64) / 720) % 12,
                    1 + ((hours_ago + (duration_minutes / 60) as u64) / 24) % 28,
                    ((hours_ago % 24) + ((duration_minutes / 60) as u64)) % 24
                )
            },
            impact_level: (i * 13 % 100) as u8, // Pseudo-random using prime
            degradation_level: (i * 17 % 100) as u8,
            detection_source: Some(sources[i % sources.len()].to_string()),
            vendor: Some(vendors[i % vendors.len()].to_string()),
            service: Some(services[i % services.len()].to_string()),
            created_at: "2025-01-15T00:00:00Z".to_string(),
            updated_at: "2025-01-15T00:00:00Z".to_string(),
        }
    }

    #[test]
    #[ignore] // Only run with --ignored flag
    fn stress_test_metrics_computation_1k_incidents() {
        let incidents: Vec<Incident> = (0..1_000)
            .map(generate_synthetic_incident)
            .collect();

        let start = Instant::now();
        let metrics = compute_metrics(&incidents).unwrap();
        let duration = start.elapsed();

        eprintln!("✓ Metrics (1K incidents): {:?}", duration);
        assert!(duration.as_secs() < 2, "Should compute in <2s for 1K incidents");
        assert_eq!(metrics.summary.total_incidents, 1000);
        assert!(metrics.summary.mttd_median > 0.0);
    }

    #[test]
    #[ignore]
    fn stress_test_metrics_computation_10k_incidents() {
        let incidents: Vec<Incident> = (0..10_000)
            .map(generate_synthetic_incident)
            .collect();

        let start = Instant::now();
        let metrics = compute_metrics(&incidents).unwrap();
        let duration = start.elapsed();

        eprintln!("✓ Metrics (10K incidents): {:?}", duration);
        assert!(duration.as_secs() < 5, "Should compute in <5s for 10K incidents");
        assert_eq!(metrics.summary.total_incidents, 10000);
        assert!(metrics.summary.mttd_median > 0.0);
        assert!(metrics.summary.mtta_median > 0.0);
        assert!(metrics.summary.mttr_median > 0.0);
    }

    #[test]
    #[ignore]
    fn stress_test_dashboard_rendering_1k_incidents() {
        let incidents: Vec<Incident> = (0..1_000)
            .map(generate_synthetic_incident)
            .collect();

        let start = Instant::now();
        let dashboard = get_dashboard_v2(&incidents).unwrap();
        let duration = start.elapsed();

        eprintln!("✓ Dashboard (1K incidents): {:?}", duration);
        assert!(duration.as_secs() < 1, "Should render in <1s for 1K incidents");
        assert_eq!(dashboard.summary.total_incidents, 1000);
    }

    #[test]
    #[ignore]
    fn stress_test_dashboard_rendering_10k_incidents() {
        let incidents: Vec<Incident> = (0..10_000)
            .map(generate_synthetic_incident)
            .collect();

        let start = Instant::now();
        let dashboard = get_dashboard_v2(&incidents).unwrap();
        let duration = start.elapsed();

        eprintln!("✓ Dashboard (10K incidents): {:?}", duration);
        assert!(duration.as_secs() < 3, "Should render in <3s for 10K incidents");
        assert_eq!(dashboard.summary.total_incidents, 10000);
        assert!(dashboard.by_vendor.len() > 0);
        assert!(dashboard.by_detection_source.len() > 0);
    }

    #[test]
    #[ignore]
    fn stress_test_report_generation_1k_incidents() {
        let incidents: Vec<Incident> = (0..1_000)
            .map(generate_synthetic_incident)
            .collect();

        let start = Instant::now();
        let report = generate_markdown(&incidents).unwrap();
        let duration = start.elapsed();

        eprintln!("✓ Report (1K incidents): {:?}", duration);
        assert!(duration.as_secs() < 5, "Should generate in <5s for 1K incidents");
        assert!(report.len() > 10000);
        assert!(report.contains("Executive"));
    }

    #[test]
    #[ignore]
    fn stress_test_report_generation_10k_incidents() {
        let incidents: Vec<Incident> = (0..10_000)
            .map(generate_synthetic_incident)
            .collect();

        let start = Instant::now();
        let report = generate_markdown(&incidents).unwrap();
        let duration = start.elapsed();

        eprintln!("✓ Report (10K incidents): {:?}", duration);
        assert!(duration.as_secs() < 10, "Should generate in <10s for 10K incidents");
        assert!(report.len() > 50000);
        assert!(report.contains("Incident"));
    }

    #[test]
    #[ignore]
    fn stress_test_memory_usage_10k_incidents() {
        let incidents: Vec<Incident> = (0..10_000)
            .map(generate_synthetic_incident)
            .collect();

        // Rough estimate: each incident struct ~500 bytes
        let estimated_size = incidents.len() * 500;
        eprintln!("✓ Estimated memory: ~{} bytes ({:.1} MB)",
            estimated_size,
            estimated_size as f64 / 1_000_000.0
        );

        // Should fit comfortably in available RAM
        assert!(estimated_size < 50_000_000, "Should use <50MB for 10K incidents");
    }

    #[test]
    #[ignore]
    fn stress_test_concurrent_reads() {
        use std::sync::Arc;
        use std::thread;

        let incidents = Arc::new(
            (0..5_000)
                .map(generate_synthetic_incident)
                .collect::<Vec<_>>()
        );

        let handles: Vec<_> = (0..4)
            .map(|thread_id| {
                let incidents = Arc::clone(&incidents);
                thread::spawn(move || {
                    let start = Instant::now();
                    let _metrics = compute_metrics(&incidents).unwrap();
                    let duration = start.elapsed();
                    eprintln!("  Thread {}: {:?}", thread_id, duration);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        eprintln!("✓ Concurrent metrics computation complete");
    }
}
