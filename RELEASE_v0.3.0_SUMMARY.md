# Release v0.3.0 - Async Mutex Thread Safety Fix

## Release Date
2025-10-29

## Version
0.3.0

## Summary
This release fixes critical thread safety issues with async operations by replacing `std::sync::Mutex` with `tokio::sync::Mutex`. All cache operations are now fully async-safe and compatible with the Tokio runtime.

## What's New

### 🔧 Critical Bug Fix
- **Issue**: `std::sync::Mutex` guards could not safely cross `.await` points, causing compilation errors in async contexts
- **Solution**: Replaced all `std::sync::Mutex` with `tokio::sync::Mutex`
- **Impact**: All cache operations are now fully async-safe

### ⚠️ Breaking Changes

All cache-related methods are now async and require `.await`:

```rust
// Before (0.2.0)
let cached = cache.get(&key);
cache.insert(key, value);
client.clear_cache();
let size = client.cache_size();

// After (0.3.0)
let cached = cache.get(&key).await;
cache.insert(key, value).await;
client.clear_cache().await;
let size = client.cache_size().await;
```

### Modified APIs

#### ResponseCache
- `pub async fn get(&self, key: &str) -> Option<String>`
- `pub async fn insert(&self, key: String, value: String)`
- `pub async fn clear(&self)`
- `pub async fn len(&self) -> usize`
- `pub async fn is_empty(&self) -> bool`

#### GenericCache<T>
- `pub async fn get(&self, key: &str) -> Option<T>`
- `pub async fn insert(&self, key: String, value: T)`
- `pub async fn clear(&self)`
- `pub async fn len(&self) -> usize`
- `pub async fn is_empty(&self) -> bool`

#### DlsiteClient
- `pub async fn clear_cache(&self)`
- `pub async fn cache_size(&self) -> usize`

## Files Changed

1. **Cargo.toml**
   - Updated version to 0.3.0
   - Added `sync` feature to tokio dependency

2. **src/cache.rs**
   - Replaced `std::sync::Mutex` with `tokio::sync::Mutex`
   - Converted all methods to async
   - Updated all tests to use `#[tokio::test]`

3. **src/client/search/mod.rs**
   - Updated cache operations to use `.await`
   - Fixed documentation test imports

4. **src/client/mod.rs**
   - Updated cache operations to use `.await`
   - Made `clear_cache()` and `cache_size()` async

5. **Documentation Files**
   - Fixed all documentation test imports to use `dlsite_gamebox` crate name
   - Updated CHANGELOG.md with breaking changes and migration guide

## Testing

✅ **All Tests Passing**
- 27 unit tests: PASS
- 5 documentation tests: PASS
- Total: 32/32 tests passing

## Migration Guide

### For Users of ResponseCache

```rust
// Old code
let cache = ResponseCache::new(100, Duration::from_secs(3600));
cache.insert("key".to_string(), "value".to_string());
if let Some(value) = cache.get("key") {
    println!("{}", value);
}

// New code
let cache = ResponseCache::new(100, Duration::from_secs(3600));
cache.insert("key".to_string(), "value".to_string()).await;
if let Some(value) = cache.get("key").await {
    println!("{}", value);
}
```

### For Users of GenericCache

```rust
// Old code
let cache = GenericCache::new(100, Duration::from_secs(3600));
cache.insert("key".to_string(), data);
if let Some(cached_data) = cache.get("key") {
    // use cached_data
}

// New code
let cache = GenericCache::new(100, Duration::from_secs(3600));
cache.insert("key".to_string(), data).await;
if let Some(cached_data) = cache.get("key").await {
    // use cached_data
}
```

### For Users of DlsiteClient

```rust
// Old code
let client = DlsiteClient::default();
client.clear_cache();
let size = client.cache_size();

// New code
let client = DlsiteClient::default();
client.clear_cache().await;
let size = client.cache_size().await;
```

## GitHub Release
- **Tag**: v0.3.0
- **Commit**: 6c037b7
- **Repository**: https://github.com/SuperToolman/dlsite-gamebox

## Cargo.io
- **Package**: https://crates.io/crates/dlsite-gamebox
- **Version**: 0.3.0

## Advantages of This Release

✅ **Thread Safety**: Fully compatible with async/await patterns
✅ **Tokio Integration**: Seamless integration with Tokio runtime
✅ **No Blocking**: All operations are non-blocking
✅ **Better Performance**: Async-safe operations enable better concurrency
✅ **Future-Proof**: Ready for streaming and advanced async features

## Known Issues

None at this time.

## Next Steps

- Implement streaming API for memory-efficient large result processing
- Add query parameter optimization with pre-compiled selectors
- Performance benchmarking and optimization

## Support

For issues or questions, please visit:
- GitHub Issues: https://github.com/SuperToolman/dlsite-gamebox/issues
- Crates.io: https://crates.io/crates/dlsite-gamebox

