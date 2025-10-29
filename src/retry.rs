use std::time::Duration;
use crate::error::DlsiteError;

/// Retry configuration for HTTP requests
#[derive(Clone, Debug)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier (exponential backoff)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_retries: u32, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay,
            backoff_multiplier: 2.0,
        }
    }

    /// Calculate the delay for a given retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);
        let delay_ms = delay_ms.min(self.max_delay.as_millis() as f64);
        Duration::from_millis(delay_ms as u64)
    }

    /// Check if an error is retryable
    pub fn is_retryable(&self, error: &DlsiteError) -> bool {
        match error {
            // Timeout errors are retryable
            DlsiteError::Timeout => true,
            // Rate limit errors are retryable
            DlsiteError::RateLimit(_) => true,
            // HTTP 5xx errors are retryable
            DlsiteError::HttpStatus(code) => *code >= 500,
            // Other errors are not retryable
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(10));
    }

    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig::default();
        let delay_0 = config.calculate_delay(0);
        let delay_1 = config.calculate_delay(1);
        let delay_2 = config.calculate_delay(2);
        
        assert_eq!(delay_0, Duration::from_millis(100));
        assert_eq!(delay_1, Duration::from_millis(200));
        assert_eq!(delay_2, Duration::from_millis(400));
    }

    #[test]
    fn test_calculate_delay_max_cap() {
        let config = RetryConfig::new(
            3,
            Duration::from_millis(100),
            Duration::from_secs(1),
        );
        let delay_10 = config.calculate_delay(10);
        assert!(delay_10 <= Duration::from_secs(1));
    }

    #[test]
    fn test_is_retryable() {
        let config = RetryConfig::default();
        
        assert!(config.is_retryable(&DlsiteError::Timeout));
        assert!(config.is_retryable(&DlsiteError::RateLimit("test".to_string())));
        assert!(config.is_retryable(&DlsiteError::HttpStatus(500)));
        assert!(config.is_retryable(&DlsiteError::HttpStatus(503)));
        
        assert!(!config.is_retryable(&DlsiteError::HttpStatus(404)));
        assert!(!config.is_retryable(&DlsiteError::HttpStatus(400)));
    }
}

