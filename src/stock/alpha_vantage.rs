use super::provider::{
    ProviderConfig, StockDataError, StockDataProvider, StockQuote,
};
use alpha_vantage::api::ApiClient;
use async_trait::async_trait;
use chrono::Utc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Rate limiting state for Alpha Vantage API
#[derive(Debug)]
struct RateLimitState {
    requests_made: u32,
    window_start: Instant,
    requests_per_minute: u32,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            requests_made: 0,
            window_start: Instant::now(),
            requests_per_minute: 5, // Alpha Vantage free tier: 5 requests per minute
        }
    }
}


/// Alpha Vantage stock data provider using the `alpha_vantage` crate
pub struct AlphaVantageProvider {
    client: Option<ApiClient>,
    rate_limit: Mutex<RateLimitState>,
}

impl AlphaVantageProvider {
    /// Create a new Alpha Vantage provider
    pub fn new() -> Self {
        Self {
            client: None,
            rate_limit: Mutex::new(RateLimitState::default()),
        }
    }

    /// Check and enforce rate limits
    async fn check_rate_limit(&self) -> Result<(), StockDataError> {
        let mut rate_limit = self.rate_limit.lock().await;
        let now = Instant::now();

        // Reset window if more than a minute has passed
        if now.duration_since(rate_limit.window_start) >= Duration::from_secs(60) {
            rate_limit.requests_made = 0;
            rate_limit.window_start = now;
        }

        // Check if we've exceeded the rate limit
        if rate_limit.requests_made >= rate_limit.requests_per_minute {
            let wait_time = Duration::from_secs(60) - now.duration_since(rate_limit.window_start);
            log::warn!("Rate limit exceeded, would need to wait {wait_time:?}");
            return Err(StockDataError::RateLimitExceeded);
        }

        rate_limit.requests_made += 1;
        Ok(())
    }

    /// Get the client, ensuring it's initialized
    fn get_client(&self) -> Result<&ApiClient, StockDataError> {
        self.client
            .as_ref()
            .ok_or_else(|| StockDataError::ConfigError("Provider not initialized".to_string()))
    }

}

#[async_trait]
impl StockDataProvider for AlphaVantageProvider {
    fn name(&self) -> &str {
        "Alpha Vantage"
    }

    async fn initialize(&mut self, config: ProviderConfig) -> Result<(), StockDataError> {
        if config.api_key.is_empty() {
            return Err(StockDataError::InvalidApiKey(
                "API key is required".to_string(),
            ));
        }

        // Update rate limit if provided
        if let Some(rate_limit) = config.rate_limit {
            let mut rl = self.rate_limit.lock().await;
            rl.requests_per_minute = rate_limit;
        }

        // Create Alpha Vantage client with reqwest client
        let http_client = reqwest::Client::new();
        self.client = Some(ApiClient::set_api(&config.api_key, http_client));

        log::info!("Alpha Vantage provider initialized successfully");
        Ok(())
    }

    async fn get_quote(&self, symbol: &str) -> Result<StockQuote, StockDataError> {
        self.check_rate_limit().await?;
        
        let client = self.get_client()?;
        
        log::debug!("Fetching quote for symbol: {}", symbol);
        
        // Use the alpha_vantage crate to get quote data
        let quote = client.quote(symbol).json().await?;
        
        // Convert the alpha_vantage quote to our StockQuote format
        Ok(StockQuote {
            symbol: quote.symbol().to_uppercase(),
            price: quote.price(),
            change: quote.change(),
            change_percent: quote.change_percent(),
            previous_close: quote.previous(),
            open: quote.open(),
            high: quote.high(),
            low: quote.low(),
            volume: quote.volume(),
            market_cap: None, // Not provided by this endpoint
            timestamp: Utc::now(), // Use current time since alpha_vantage doesn't provide exact timestamp
        })
    }

    async fn get_news(&self, _symbol: &str, _limit: usize) -> Result<Vec<crate::stock::provider::StockNews>, StockDataError> {
        // TODO: Implement news API when alpha_vantage crate supports it
        Ok(Vec::new())
    }

    async fn get_market_news(&self, _limit: usize) -> Result<Vec<crate::stock::provider::StockNews>, StockDataError> {
        // TODO: Implement market news API when alpha_vantage crate supports it
        Ok(Vec::new())
    }

    async fn validate_symbol(&self, symbol: &str) -> Result<bool, StockDataError> {
        // Try to fetch a quote for the symbol
        match self.get_quote(symbol).await {
            Ok(_) => Ok(true),
            Err(StockDataError::SymbolNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn get_rate_limit_info(&self) -> Option<(u32, u32)> {
        // We can't access the mutex synchronously, so return None
        // In a real implementation, you might want to track this separately
        None
    }

    async fn health_check(&self) -> Result<(), StockDataError> {
        // Test with a known symbol
        self.get_quote("AAPL").await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AlphaVantageProvider::new();
        assert_eq!(provider.name(), "Alpha Vantage");
        assert!(provider.client.is_none());
    }

    #[test]
    fn test_rate_limit_state() {
        let state = RateLimitState::default();
        assert_eq!(state.requests_made, 0);
        assert_eq!(state.requests_per_minute, 5);
    }

    #[tokio::test]
    async fn test_initialization_with_empty_api_key() {
        let mut provider = AlphaVantageProvider::new();
        let config = ProviderConfig {
            api_key: String::new(),
            ..Default::default()
        };
        
        let result = provider.initialize(config).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StockDataError::InvalidApiKey(_)));
    }

    #[tokio::test]
    async fn test_initialization_with_valid_config() {
        let mut provider = AlphaVantageProvider::new();
        let config = ProviderConfig {
            api_key: "test_key".to_string(),
            rate_limit: Some(10),
            ..Default::default()
        };
        
        let result = provider.initialize(config).await;
        assert!(result.is_ok());
        assert!(provider.client.is_some());
    }

    #[tokio::test]
    async fn test_get_quote_without_initialization() {
        let provider = AlphaVantageProvider::new();
        let result = provider.get_quote("AAPL").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StockDataError::ConfigError(_)));
    }
}