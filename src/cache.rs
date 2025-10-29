use std::sync::Arc;
use std::time::{Duration, Instant};
use lru::LruCache;
use std::sync::Mutex;
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
    pub fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.lock().unwrap();
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
    pub fn insert(&self, key: String, value: String) {
        let mut cache = self.cache.lock().unwrap();
        let entry = CacheEntry {
            data: value,
            expires_at: Instant::now() + self.ttl,
        };
        cache.put(key, entry);
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.lock().unwrap();
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
    pub fn get(&self, key: &str) -> Option<T> {
        let mut cache = self.cache.lock().unwrap();
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
    pub fn insert(&self, key: String, value: T) {
        let mut cache = self.cache.lock().unwrap();
        let entry = CacheEntry {
            data: value,
            expires_at: Instant::now() + self.ttl,
        };
        cache.put(key, entry);
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.lock().unwrap();
        cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        let cache = ResponseCache::new(10, Duration::from_secs(60));
        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_cache_expiration() {
        let cache = ResponseCache::new(10, Duration::from_millis(100));
        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = ResponseCache::new(2, Duration::from_secs(60));
        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        cache.insert("key3".to_string(), "value3".to_string());
        
        // key1 should be evicted
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), Some("value2".to_string()));
        assert_eq!(cache.get("key3"), Some("value3".to_string()));
    }

    #[test]
    fn test_cache_clear() {
        let cache = ResponseCache::new(10, Duration::from_secs(60));
        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        assert_eq!(cache.len(), 2);
        
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
}

