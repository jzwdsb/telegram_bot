use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Error types for stock data operations
#[derive(Debug)]
pub enum StockDataError {
    /// API key is missing or invalid
    InvalidApiKey(String),
    /// Network request failed
    NetworkError(String),
    /// Failed to parse response
    ParseError(String),
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Stock symbol not found
    SymbolNotFound(String),
    /// Invalid stock symbol
    InvalidSymbol(String),
    /// Provider-specific error
    ProviderError(String),
    /// Configuration error
    ConfigError(String),
}

impl fmt::Display for StockDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StockDataError::InvalidApiKey(msg) => write!(f, "Invalid API key: {}", msg),
            StockDataError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            StockDataError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            StockDataError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            StockDataError::SymbolNotFound(symbol) => write!(f, "Symbol not found: {}", symbol),
            StockDataError::InvalidSymbol(symbol) => write!(f, "Invalid symbol: {}", symbol),
            StockDataError::ProviderError(msg) => write!(f, "Provider error: {}", msg),
            StockDataError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl Error for StockDataError {}

/// Convert AlphaVantageError to StockDataError
impl From<alpha_vantage::error::Error> for StockDataError {
    fn from(error: alpha_vantage::error::Error) -> Self {
        let error_msg = format!("{:?}", error);
        
        // Parse common error patterns from the error message
        if error_msg.contains("Invalid API call") || error_msg.contains("symbol") || error_msg.contains("InvalidData") {
            StockDataError::SymbolNotFound(error_msg)
        } else if error_msg.contains("API key") {
            StockDataError::InvalidApiKey(error_msg)
        } else if error_msg.contains("call frequency") || error_msg.contains("premium") {
            StockDataError::RateLimitExceeded
        } else if error_msg.contains("network") || error_msg.contains("connection") {
            StockDataError::NetworkError(error_msg)
        } else {
            StockDataError::ProviderError(error_msg)
        }
    }
}

/// Stock quote data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    /// Stock symbol (e.g., "AAPL")
    pub symbol: String,
    /// Current price
    pub price: f64,
    /// Price change from previous close
    pub change: f64,
    /// Percentage change from previous close
    pub change_percent: f64,
    /// Previous closing price
    pub previous_close: f64,
    /// Opening price
    pub open: f64,
    /// Daily high
    pub high: f64,
    /// Daily low
    pub low: f64,
    /// Trading volume
    pub volume: u64,
    /// Market cap (optional)
    pub market_cap: Option<u64>,
    /// Last update timestamp
    pub timestamp: DateTime<Utc>,
}

/// News sentiment classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

/// Stock news article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockNews {
    /// Article title
    pub title: String,
    /// Article summary or content
    pub summary: String,
    /// Source of the news
    pub source: String,
    /// Publication timestamp
    pub published_at: DateTime<Utc>,
    /// URL to the full article
    pub url: String,
    /// Related stock symbols
    pub symbols: Vec<String>,
    /// Sentiment analysis (optional)
    pub sentiment: Option<Sentiment>,
    /// AI-generated summary (will be populated later)
    pub ai_summary: Option<String>,
}

/// Configuration for stock data providers
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// API key for the provider
    pub api_key: String,
    /// Base URL for API requests (optional override)
    pub base_url: Option<String>,
    /// Request timeout in seconds
    pub timeout: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
    /// Rate limit (requests per minute)
    pub rate_limit: Option<u32>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: None,
            timeout: 30,
            max_retries: 3,
            rate_limit: None,
        }
    }
}

/// Trait for stock data providers
#[async_trait]
pub trait StockDataProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Initialize the provider with configuration
    async fn initialize(&mut self, config: ProviderConfig) -> Result<(), StockDataError>;

    /// Fetch current quote for a single stock
    async fn get_quote(&self, symbol: &str) -> Result<StockQuote, StockDataError>;

    /// Fetch quotes for multiple stocks (batch operation)
    async fn get_quotes(&self, symbols: &[String]) -> Result<Vec<StockQuote>, StockDataError> {
        // Default implementation: fetch quotes individually
        let mut quotes = Vec::new();
        for symbol in symbols {
            match self.get_quote(symbol).await {
                Ok(quote) => quotes.push(quote),
                Err(e) => {
                    log::warn!("Failed to fetch quote for {}: {}", symbol, e);
                    // Continue with other symbols instead of failing entirely
                }
            }
        }
        
        if quotes.is_empty() && !symbols.is_empty() {
            return Err(StockDataError::ProviderError(
                "Failed to fetch any quotes".to_string(),
            ));
        }
        
        Ok(quotes)
    }

    /// Fetch recent news for a stock
    async fn get_news(
        &self,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<StockNews>, StockDataError>;

    /// Fetch market news (not specific to a symbol)
    async fn get_market_news(&self, limit: usize) -> Result<Vec<StockNews>, StockDataError>;

    /// Validate a stock symbol
    async fn validate_symbol(&self, symbol: &str) -> Result<bool, StockDataError> {
        // Default implementation: try to fetch a quote
        match self.get_quote(symbol).await {
            Ok(_) => Ok(true),
            Err(StockDataError::SymbolNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get rate limit information
    fn get_rate_limit_info(&self) -> Option<(u32, u32)> {
        // Returns (used, limit) if available
        None
    }

    /// Check if provider is healthy and accessible
    async fn health_check(&self) -> Result<(), StockDataError> {
        // Default implementation: try to fetch a known symbol
        self.get_quote("AAPL").await.map(|_| ())
    }
}

/// Factory for creating stock data providers
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a provider by name
    pub fn create(provider_type: &str) -> Result<Box<dyn StockDataProvider>, StockDataError> {
        match provider_type.to_lowercase().as_str() {
            "alpha_vantage" | "alphavantage" => {
                use crate::stock::alpha_vantage::AlphaVantageProvider;
                Ok(Box::new(AlphaVantageProvider::new()))
            }
            _ => Err(StockDataError::ConfigError(format!(
                "Unknown provider type: {}",
                provider_type
            ))),
        }
    }

    /// List available providers
    pub fn available_providers() -> Vec<&'static str> {
        vec!["alpha_vantage"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_quote_creation() {
        let quote = StockQuote {
            symbol: "AAPL".to_string(),
            price: 150.0,
            change: 2.5,
            change_percent: 1.69,
            previous_close: 147.5,
            open: 148.0,
            high: 151.0,
            low: 147.0,
            volume: 50_000_000,
            market_cap: Some(2_500_000_000_000),
            timestamp: Utc::now(),
        };

        assert_eq!(quote.symbol, "AAPL");
        assert_eq!(quote.price, 150.0);
        assert_eq!(quote.change_percent, 1.69);
    }

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.base_url.is_none());
        assert!(config.rate_limit.is_none());
    }

    #[test]
    fn test_error_display() {
        let error = StockDataError::SymbolNotFound("INVALID".to_string());
        assert_eq!(format!("{}", error), "Symbol not found: INVALID");

        let error = StockDataError::RateLimitExceeded;
        assert_eq!(format!("{}", error), "Rate limit exceeded");
    }

    #[test]
    fn test_provider_factory_create_alpha_vantage() {
        let provider = ProviderFactory::create("alpha_vantage");
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_factory_create_unknown() {
        let provider = ProviderFactory::create("unknown");
        assert!(provider.is_err());
    }

    #[test]
    fn test_provider_factory_available_providers() {
        let providers = ProviderFactory::available_providers();
        assert!(providers.contains(&"alpha_vantage"));
    }

    #[test]
    fn test_alpha_vantage_error_conversion() {
        // Create a mock alpha_vantage error and test conversion
        use alpha_vantage::error::Error as AlphaVantageError;
        
        // Test that we can convert alpha_vantage errors to our error type
        let alpha_error = AlphaVantageError::AlphaVantageErrorMessage("Invalid API call".to_string());
        let stock_error: StockDataError = alpha_error.into();
        
        match stock_error {
            StockDataError::SymbolNotFound(_) => {}, // Expected
            _ => panic!("Expected SymbolNotFound error"),
        }
    }
}