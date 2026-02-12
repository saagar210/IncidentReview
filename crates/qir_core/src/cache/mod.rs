/// Caching layer for expensive computations (dashboards, metrics)
///
/// Provides:
/// - Dashboard caching with 5-minute TTL
/// - Automatic invalidation on data mutation
/// - Content hash-based cache keys
/// - Thread-safe access via Arc<Mutex<>>

use std::sync::Mutex;
use std::time::{SystemTime, Duration};
use crate::analytics::{DashboardPayloadV1, DashboardPayloadV2};

/// Cached dashboard payload with timestamp and hash
struct CachedDashboard<T> {
    data: T,
    computed_at: SystemTime,
    data_hash: String, // SHA256 of incidents (for cache invalidation)
}

/// Dashboard cache with TTL-based expiration
pub struct DashboardCache {
    v1: Mutex<Option<CachedDashboard<DashboardPayloadV1>>>,
    v2: Mutex<Option<CachedDashboard<DashboardPayloadV2>>>,
    ttl_seconds: u64,
}

impl DashboardCache {
    /// Create a new dashboard cache with 5-minute TTL
    pub fn new() -> Self {
        DashboardCache {
            v1: Mutex::new(None),
            v2: Mutex::new(None),
            ttl_seconds: 300, // 5 minutes
        }
    }

    /// Create a cache with custom TTL (for testing)
    pub fn with_ttl(ttl_seconds: u64) -> Self {
        DashboardCache {
            v1: Mutex::new(None),
            v2: Mutex::new(None),
            ttl_seconds,
        }
    }

    /// Get cached DashboardPayloadV1 if valid (hash matches, not expired)
    pub fn get_v1(&self, current_hash: &str) -> Option<DashboardPayloadV1> {
        let cache = self.v1.lock().unwrap();
        if let Some(cached) = cache.as_ref() {
            // Check if hash matches (data hasn't changed)
            if cached.data_hash != current_hash {
                return None;
            }

            // Check if still valid (not expired)
            let age = SystemTime::now()
                .duration_since(cached.computed_at)
                .unwrap_or(Duration::from_secs(self.ttl_seconds + 1));

            if age.as_secs() < self.ttl_seconds {
                return Some(cached.data.clone());
            }
        }
        None
    }

    /// Get cached DashboardPayloadV2 if valid
    pub fn get_v2(&self, current_hash: &str) -> Option<DashboardPayloadV2> {
        let cache = self.v2.lock().unwrap();
        if let Some(cached) = cache.as_ref() {
            // Check hash
            if cached.data_hash != current_hash {
                return None;
            }

            // Check expiration
            let age = SystemTime::now()
                .duration_since(cached.computed_at)
                .unwrap_or(Duration::from_secs(self.ttl_seconds + 1));

            if age.as_secs() < self.ttl_seconds {
                return Some(cached.data.clone());
            }
        }
        None
    }

    /// Store DashboardPayloadV1 in cache
    pub fn set_v1(&self, dashboard: DashboardPayloadV1, hash: String) {
        let mut cache = self.v1.lock().unwrap();
        *cache = Some(CachedDashboard {
            data: dashboard,
            computed_at: SystemTime::now(),
            data_hash: hash,
        });
    }

    /// Store DashboardPayloadV2 in cache
    pub fn set_v2(&self, dashboard: DashboardPayloadV2, hash: String) {
        let mut cache = self.v2.lock().unwrap();
        *cache = Some(CachedDashboard {
            data: dashboard,
            computed_at: SystemTime::now(),
            data_hash: hash,
        });
    }

    /// Invalidate all cached dashboards (call on data mutation)
    pub fn invalidate_all(&self) {
        *self.v1.lock().unwrap() = None;
        *self.v2.lock().unwrap() = None;
    }

    /// Invalidate only V1 cache
    pub fn invalidate_v1(&self) {
        *self.v1.lock().unwrap() = None;
    }

    /// Invalidate only V2 cache
    pub fn invalidate_v2(&self) {
        *self.v2.lock().unwrap() = None;
    }

    /// Get cache statistics (for monitoring)
    pub fn stats(&self) -> CacheStats {
        let v1_cached = self.v1.lock().unwrap().is_some();
        let v2_cached = self.v2.lock().unwrap().is_some();

        CacheStats {
            v1_cached,
            v2_cached,
            ttl_seconds: self.ttl_seconds,
        }
    }
}

impl Default for DashboardCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub v1_cached: bool,
    pub v2_cached: bool,
    pub ttl_seconds: u64,
}

/// Compute SHA256 hash of incidents for cache invalidation
pub fn compute_incidents_hash(incident_ids: &[String]) -> String {
    use sha2::{Sha256, Digest};
    use hex::encode;

    let mut hasher = Sha256::new();
    for id in incident_ids {
        hasher.update(id.as_bytes());
    }
    encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cache_hit() {
        let cache = DashboardCache::new();
        let hash = "abc123".to_string();

        // Create dummy dashboard
        let dashboard = DashboardPayloadV1 {
            version: 1,
            incident_count: 0,
            severity_counts: vec![],
            incidents: vec![],
        };

        cache.set_v1(dashboard.clone(), hash.clone());
        let result = cache.get_v1(&hash);

        assert!(result.is_some());
        assert_eq!(result.unwrap().incident_count, dashboard.incident_count);
    }

    #[test]
    fn test_cache_miss_on_hash_mismatch() {
        let cache = DashboardCache::new();
        let hash1 = "abc123".to_string();
        let hash2 = "def456".to_string();

        let dashboard = DashboardPayloadV1 {
            version: 1,
            incident_count: 0,
            severity_counts: vec![],
            incidents: vec![],
        };
        cache.set_v1(dashboard, hash1);

        // Different hash should result in miss
        let result = cache.get_v1(&hash2);
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_expiration() {
        let cache = DashboardCache::with_ttl(1); // 1 second TTL
        let hash = "abc123".to_string();

        let dashboard = DashboardPayloadV1 {
            version: 1,
            incident_count: 0,
            severity_counts: vec![],
            incidents: vec![],
        };
        cache.set_v1(dashboard, hash.clone());

        // Immediate access should hit
        assert!(cache.get_v1(&hash).is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(1100));

        // Should be expired
        assert!(cache.get_v1(&hash).is_none());
    }

    #[test]
    fn test_cache_invalidate_all() {
        let cache = DashboardCache::new();
        let hash = "abc123".to_string();

        let v1 = DashboardPayloadV1 {
            version: 1,
            incident_count: 0,
            severity_counts: vec![],
            incidents: vec![],
        };
        let v2 = DashboardPayloadV2 {
            version: 2,
            incident_count: 0,
            severity_counts: vec![],
            incidents: vec![],
            detection_story: Default::default(),
            vendor_service_story: Default::default(),
            response_story: Default::default(),
        };

        cache.set_v1(v1, hash.clone());
        cache.set_v2(v2, hash.clone());

        assert!(cache.get_v1(&hash).is_some());
        assert!(cache.get_v2(&hash).is_some());

        cache.invalidate_all();

        assert!(cache.get_v1(&hash).is_none());
        assert!(cache.get_v2(&hash).is_none());
    }

    #[test]
    fn test_compute_incidents_hash() {
        let ids = vec![
            "incident-1".to_string(),
            "incident-2".to_string(),
        ];

        let hash = compute_incidents_hash(&ids);
        let hash2 = compute_incidents_hash(&ids);

        // Same input should produce same hash
        assert_eq!(hash, hash2);

        // Different input should produce different hash
        let ids2 = vec!["incident-3".to_string()];
        let hash3 = compute_incidents_hash(&ids2);
        assert_ne!(hash, hash3);
    }
}
