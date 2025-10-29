use std::sync::Arc;
use std::time::{Duration, Instant};
use lru::LruCache;
use tokio::sync::Mutex;
use std::num::NonZeroUsize;

/// Generic cache entry with expiration time
#[derive(Clone, Debug)]
struct CacheEntry<T: Clone> {
    data: T,
    expires_at: Instant,
}

impl<T: Clone> CacheEntry<T> {
    /// Check if the cache entry has expired
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Thread-safe LRU cache for HTTP responses
#[derive(Clone, Debug)]
pub struct ResponseCache {
    cache: Arc<Mutex<LruCache<String, CacheEntry<String>>>>,
    ttl: Duration,
}

impl ResponseCache {
    /// Create a new response cache with the specified capacity and TTL
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of entries in the cache
    /// * `ttl` - Time to live for each cache entry
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(capacity).unwrap());
        Self {
            cache: Arc::new(Mutex::new(cache)),
            ttl,
        }
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.lock().await;
        if let Some(entry) = cache.get_mut(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            } else {
                // Remove expired entry
                cache.pop(key);
            }
        }
        None
    }

    /// Insert a value into the cache
    pub async fn insert(&self, key: String, value: String) {
        let mut cache = self.cache.lock().await;
        let entry = CacheEntry {
            data: value,
            expires_at: Instant::now() + self.ttl,
        };
        cache.put(key, entry);
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    /// Get the number of entries in the cache
    pub async fn len(&self) -> usize {
        let cache = self.cache.lock().await;
        cache.len()
    }

    /// Check if the cache is empty
    pub async fn is_empty(&self) -> bool {
        let cache = self.cache.lock().await;
        cache.is_empty()
    }
}

/// Generic thread-safe LRU cache for any type of data
#[derive(Clone, Debug)]
pub struct GenericCache<T: Clone> {
    cache: Arc<Mutex<LruCache<String, CacheEntry<T>>>>,
    ttl: Duration,
}

impl<T: Clone> GenericCache<T> {
    /// Create a new generic cache with the specified capacity and TTL
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(capacity).unwrap());
        Self {
            cache: Arc::new(Mutex::new(cache)),
            ttl,
        }
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &str) -> Option<T> {
        let mut cache = self.cache.lock().await;
        if let Some(entry) = cache.get_mut(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            } else {
                cache.pop(key);
            }
        }
        None
    }

    /// Insert a value into the cache
    pub async fn insert(&self, key: String, value: T) {
        let mut cache = self.cache.lock().await;
        let entry = CacheEntry {
            data: value,
            expires_at: Instant::now() + self.ttl,
        };
        cache.put(key, entry);
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    /// Get the number of entries in the cache
    pub async fn len(&self) -> usize {
        let cache = self.cache.lock().await;
        cache.len()
    }

    /// Check if the cache is empty
    pub async fn is_empty(&self) -> bool {
        let cache = self.cache.lock().await;
        cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache = ResponseCache::new(10, Duration::from_secs(60));
        cache.insert("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = ResponseCache::new(10, Duration::from_millis(100));
        cache.insert("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));

        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_lru_eviction() {
        let cache = ResponseCache::new(2, Duration::from_secs(60));
        cache.insert("key1".to_string(), "value1".to_string()).await;
        cache.insert("key2".to_string(), "value2".to_string()).await;
        cache.insert("key3".to_string(), "value3".to_string()).await;

        // key1 should be evicted
        assert_eq!(cache.get("key1").await, None);
        assert_eq!(cache.get("key2").await, Some("value2".to_string()));
        assert_eq!(cache.get("key3").await, Some("value3".to_string()));
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = ResponseCache::new(10, Duration::from_secs(60));
        cache.insert("key1".to_string(), "value1".to_string()).await;
        cache.insert("key2".to_string(), "value2".to_string()).await;
        assert_eq!(cache.len().await, 2);

        cache.clear().await;
        assert_eq!(cache.len().await, 0);
        assert!(cache.is_empty().await);
    }
}

