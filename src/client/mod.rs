use crate::cache::ResponseCache;
use crate::error::{DlsiteError, Result};
use crate::retry::RetryConfig;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

pub mod circle;
pub mod product;
pub mod product_api;
pub mod search;

/// API client for DLsite.
#[derive(Clone, Debug)]
pub struct DlsiteClient {
    client: reqwest::Client,
    base_url: String,
    /// Rate limiter to prevent IP bans (2 requests per second by default)
    /// Stores the timestamp of the last request in milliseconds
    last_request_time: Arc<AtomicU64>,
    /// Response cache for caching HTTP responses
    cache: ResponseCache,
    /// Retry configuration for automatic retries
    retry_config: RetryConfig,
}

impl Default for DlsiteClient {
    fn default() -> Self {
        Self::new("https://www.dlsite.com/maniax")
    }
}

/// Builder for DlsiteClient with customizable configuration
pub struct DlsiteClientBuilder {
    base_url: String,
    pool_max_idle_per_host: usize,
    timeout: Duration,
    cache_capacity: usize,
    cache_ttl: Duration,
    retry_config: RetryConfig,
}

impl DlsiteClientBuilder {
    /// Create a new builder with default settings
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            pool_max_idle_per_host: 10,
            timeout: Duration::from_secs(30),
            cache_capacity: 100,
            cache_ttl: Duration::from_secs(3600),
            retry_config: RetryConfig::default(),
        }
    }

    /// Set the maximum number of idle connections per host
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = max;
        self
    }

    /// Set the request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the cache capacity and TTL
    pub fn cache(mut self, capacity: usize, ttl: Duration) -> Self {
        self.cache_capacity = capacity;
        self.cache_ttl = ttl;
        self
    }

    /// Set the retry configuration
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Build the DlsiteClient
    pub fn build(self) -> DlsiteClient {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(self.pool_max_idle_per_host)
            .timeout(self.timeout)
            .user_agent("dlsite-rs/0.2.0")
            .build()
            .expect("Failed to build HTTP client");

        DlsiteClient {
            client,
            base_url: self.base_url,
            last_request_time: Arc::new(AtomicU64::new(0)),
            cache: ResponseCache::new(self.cache_capacity, self.cache_ttl),
            retry_config: self.retry_config,
        }
    }
}

impl DlsiteClient {
    /// Create a new DLsite client with a custom base URL.
    ///
    /// Typical base URL is `https://www.dlsite.com/maniax` and you should be able to access any
    /// products using this URL, so usually you don't use this method and just use the default.
    ///
    /// The client is configured with:
    /// - Connection pool: 10 idle connections per host
    /// - Timeout: 30 seconds
    /// - User-Agent: dlsite-rs/0.2.0
    /// - Rate limit: 2 requests per second
    /// - Cache: 100 entries with 1 hour TTL
    /// - Retry: 3 attempts with exponential backoff
    pub fn new(base_url: &str) -> Self {
        DlsiteClientBuilder::new(base_url).build()
    }

    /// Create a builder for customizing the client configuration
    pub fn builder(base_url: &str) -> DlsiteClientBuilder {
        DlsiteClientBuilder::new(base_url)
    }

    /// Convenient method to make a http GET request using the client.
    ///
    /// This method respects the rate limiter to prevent IP bans, uses caching, and retries on failure.
    /// Rate limit: 2 requests per second (500ms between requests)
    /// Cache: 100 entries with 1 hour TTL
    /// Retry: 3 attempts with exponential backoff for retryable errors
    pub async fn get(&self, path: &str) -> Result<String> {
        let url = format!("{}{}", self.base_url, path);

        // Check cache first
        if let Some(cached) = self.cache.get(&url) {
            return Ok(cached);
        }

        // Retry loop
        let mut last_error = None;
        for attempt in 0..=self.retry_config.max_retries {
            // Rate limiting: ensure at least 500ms between requests (2 req/sec)
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let last_time = self.last_request_time.load(std::sync::atomic::Ordering::Relaxed);
            let elapsed = now.saturating_sub(last_time);

            if elapsed < 500 {
                let sleep_time = Duration::from_millis(500 - elapsed);
                tokio::time::sleep(sleep_time).await;
            }

            self.last_request_time.store(now, std::sync::atomic::Ordering::Relaxed);

            match self.client.get(&url).send().await {
                Ok(response) => {
                    // Check HTTP status code
                    let status = response.status();
                    if status == 429 {
                        let err = DlsiteError::RateLimit(
                            "Too many requests, please retry later".to_string()
                        );
                        if attempt < self.retry_config.max_retries && self.retry_config.is_retryable(&err) {
                            last_error = Some(err);
                            let delay = self.retry_config.calculate_delay(attempt);
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(err);
                    }
                    if !status.is_success() {
                        let err = DlsiteError::HttpStatus(status.as_u16());
                        if attempt < self.retry_config.max_retries && self.retry_config.is_retryable(&err) {
                            last_error = Some(err);
                            let delay = self.retry_config.calculate_delay(attempt);
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(err);
                    }

                    let body = response.text().await?;

                    // Cache the response
                    self.cache.insert(url, body.clone());

                    return Ok(body);
                }
                Err(e) => {
                    let err = DlsiteError::from(e);
                    if attempt < self.retry_config.max_retries && self.retry_config.is_retryable(&err) {
                        last_error = Some(err);
                        let delay = self.retry_config.calculate_delay(attempt);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        // If we exhausted all retries, return the last error
        Err(last_error.unwrap_or_else(|| DlsiteError::Parse("Unknown error".to_string())))
    }

    /// Similar to `get`, but this method does not prepend the base URL.
    pub async fn get_raw(&self, url: &str) -> Result<String> {
        let body = self.client.get(url).send().await?.text().await?;
        Ok(body)
    }

    /// Clear the response cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get the number of entries in the cache
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Set the retry configuration
    pub fn set_retry_config(&mut self, config: RetryConfig) {
        self.retry_config = config;
    }

    /// Get the current retry configuration
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }
}

/// These methods return a “sub-client”.
/// The sub-client has a DlsiteClient reference inside and has implementations of fetch and parse focused on certain purposes.
impl DlsiteClient {
    /// Get a client to fetch product info using 'scraping' method. For more information, see [`product::ProductClient`].
    pub fn product(&self) -> product::ProductClient {
        product::ProductClient { c: self }
    }

    /// Get a client to fetch product info using 'api' method. For more information, see
    /// [`product_api::ProductApiClient`].
    pub fn product_api(&self) -> product_api::ProductApiClient {
        product_api::ProductApiClient { c: self }
    }

    /// Get a client to fetch circle info. For more information, see [`circle::CircleClient`].
    pub fn circle(&self) -> circle::CircleClient {
        circle::CircleClient { c: self }
    }

    /// Get a client to search things. For more information, see [`search::SearchClient`].
    pub fn search(&self) -> search::SearchClient {
        search::SearchClient::new(self)
    }
}
