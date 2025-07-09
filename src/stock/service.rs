use super::provider::{ProviderConfig, StockDataError, StockDataProvider, StockQuote};
use super::alpha_vantage::AlphaVantageProvider;
use std::env;

/// Stock service for handling stock operations
pub struct StockService {
    provider: Box<dyn StockDataProvider>,
}

impl StockService {
    /// Create a new stock service with Alpha Vantage provider
    pub async fn new() -> Result<Self, StockDataError> {
        let api_key = env::var("ALPHA_VANTAGE_API_KEY")
            .map_err(|_| StockDataError::ConfigError("ALPHA_VANTAGE_API_KEY environment variable not set".to_string()))?;

        if api_key.is_empty() {
            return Err(StockDataError::InvalidApiKey("API key is empty".to_string()));
        }

        let config = ProviderConfig {
            api_key,
            rate_limit: Some(5), // Free tier: 5 requests per minute
            base_url: None,
            timeout: 30,
            max_retries: 3,
        };

        let mut provider = AlphaVantageProvider::new();
        provider.initialize(config).await?;

        Ok(Self {
            provider: Box::new(provider),
        })
    }

    /// Get stock quote for a symbol
    pub async fn get_quote(&self, symbol: &str) -> Result<StockQuote, StockDataError> {
        if symbol.trim().is_empty() {
            return Err(StockDataError::InvalidSymbol("Symbol cannot be empty".to_string()));
        }

        let symbol = symbol.trim().to_uppercase();
        log::info!("Fetching quote for symbol: {symbol}");

        match self.provider.get_quote(&symbol).await {
            Ok(quote) => {
                log::info!("Successfully fetched quote for {}: ${:.2}", symbol, quote.price);
                Ok(quote)
            }
            Err(e) => {
                log::error!("Failed to fetch quote for {symbol}: {e:?}");
                Err(e)
            }
        }
    }

    /// Get news for a stock symbol (placeholder for now)
    pub async fn get_news(&self, symbol: &str) -> Result<String, StockDataError> {
        if symbol.trim().is_empty() {
            return Err(StockDataError::InvalidSymbol("Symbol cannot be empty".to_string()));
        }

        let symbol = symbol.trim().to_uppercase();
        log::info!("News requested for symbol: {symbol}");

        // For now, return a placeholder message since the alpha_vantage crate doesn't support news yet
        Ok(format!(
            "📰 {symbol} News\n\n🚧 News feature coming soon!\nCurrently using Alpha Vantage crate which doesn't yet support news API.\n\nFor now, try these alternatives:\n• Check financial news websites\n• Use the /price command for current stock data"
        ))
    }

    /// Validate if a stock symbol exists
    pub async fn validate_symbol(&self, symbol: &str) -> Result<bool, StockDataError> {
        self.provider.validate_symbol(symbol).await
    }

    /// Get provider health status
    pub async fn health_check(&self) -> Result<(), StockDataError> {
        self.provider.health_check().await
    }
}

/// Format stock quote for display
pub fn format_stock_quote(quote: &StockQuote) -> String {
    let symbol = &quote.symbol;
    let price = quote.price;
    let change = quote.change;
    let change_percent = quote.change_percent;
    
    // Determine emoji based on price change
    let trend_emoji = if change > 0.0 {
        "📈"
    } else if change < 0.0 {
        "📉"
    } else {
        "➡️"
    };

    // Format change with proper sign
    let change_sign = if change >= 0.0 { "+" } else { "" };
    let change_percent_sign = if change_percent >= 0.0 { "+" } else { "" };

    // Format volume in a more readable way
    let volume_str = {
        let volume = quote.volume as f64;
        if volume >= 1_000_000.0 {
            format!("{:.1}M", volume / 1_000_000.0)
        } else if volume >= 1_000.0 {
            format!("{:.1}K", volume / 1_000.0)
        } else {
            format!("{volume:.0}")
        }
    };

    // Format market cap if available
    let market_cap_str = if let Some(market_cap) = quote.market_cap {
        let market_cap = market_cap as f64;
        if market_cap >= 1_000_000_000_000.0 {
            format!("{:.1}T", market_cap / 1_000_000_000_000.0)
        } else if market_cap >= 1_000_000_000.0 {
            format!("{:.1}B", market_cap / 1_000_000_000.0)
        } else if market_cap >= 1_000_000.0 {
            format!("{:.1}M", market_cap / 1_000_000.0)
        } else {
            format!("{market_cap:.0}")
        }
    } else {
        "N/A".to_string()
    };

    let timestamp_str = quote.timestamp.format("%Y-%m-%d %H:%M UTC").to_string();

    format!(
        "{} {} Stock Quote\n\nPrice: ${:.2} (${}{:.2}, {}{}%)\nOpen: ${:.2}\nHigh: ${:.2}\nLow: ${:.2}\nVolume: {}\nMarket Cap: ${}\n\nLast Updated: {}\nData provided by Alpha Vantage",
        trend_emoji,
        symbol,
        price,
        change_sign,
        change,
        change_percent_sign,
        change_percent,
        quote.open,
        quote.high,
        quote.low,
        volume_str,
        market_cap_str,
        timestamp_str
    )
}

/// Format error messages for user display
pub fn format_stock_error(error: &StockDataError, symbol: Option<&str>) -> String {
    match error {
        StockDataError::InvalidSymbol(_) | StockDataError::SymbolNotFound(_) => {
            if let Some(symbol) = symbol {
                let upper_symbol = symbol.to_uppercase();
                let suggestion = match upper_symbol.as_str() {
                    "APPL" => "\n💡 Did you mean AAPL (Apple Inc.)?",
                    "GOOG" => "\n💡 Try GOOGL (Alphabet Inc.)",
                    "MSFT" => "\n💡 Already correct symbol",
                    _ => "\n💡 Make sure you're using the correct ticker symbol"
                };
                format!("❌ Stock symbol not found: \"{upper_symbol}\"\nPlease check the symbol and try again.{suggestion}")
            } else {
                "❌ Invalid stock symbol\nPlease provide a valid stock symbol.".to_string()
            }
        }
        StockDataError::RateLimitExceeded => {
            "⚠️ Rate limit exceeded\nPlease wait a moment before trying again.".to_string()
        }
        StockDataError::NetworkError(_) => {
            "🌐 Network error\nPlease check your connection and try again.".to_string()
        }
        StockDataError::InvalidApiKey(_) => {
            "🔑 API configuration error\nPlease contact the administrator.".to_string()
        }
        StockDataError::ConfigError(_) => {
            "⚙️ Configuration error\nPlease contact the administrator.".to_string()
        }
        _ => {
            "🔧 Service temporarily unavailable\nPlease try again later.".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_format_stock_quote() {
        let quote = StockQuote {
            symbol: "AAPL".to_string(),
            price: 150.25,
            change: 2.35,
            change_percent: 1.58,
            previous_close: Some(147.90),
            open: Some(148.90),
            high: Some(151.20),
            low: Some(147.80),
            volume: Some(45_200_000.0),
            market_cap: Some(2_400_000_000_000.0),
            timestamp: Utc::now(),
        };

        let formatted = format_stock_quote(&quote);
        
        assert!(formatted.contains("📈 AAPL Stock Quote"));
        assert!(formatted.contains("$150.25"));
        assert!(formatted.contains("+$2.35"));
        assert!(formatted.contains("+1.58%"));
        assert!(formatted.contains("45.2M"));
        assert!(formatted.contains("$2.4T"));
    }

    #[test]
    fn test_format_negative_change() {
        let quote = StockQuote {
            symbol: "MSFT".to_string(),
            price: 380.10,
            change: -1.50,
            change_percent: -0.39,
            previous_close: Some(381.60),
            open: Some(381.00),
            high: Some(382.50),
            low: Some(379.80),
            volume: Some(25_500_000.0),
            market_cap: None,
            timestamp: Utc::now(),
        };

        let formatted = format_stock_quote(&quote);
        
        assert!(formatted.contains("📉 MSFT Stock Quote"));
        assert!(formatted.contains("-$1.50"));
        assert!(formatted.contains("-0.39%"));
        assert!(formatted.contains("25.5M"));
        assert!(formatted.contains("N/A"));
    }

    #[test]
    fn test_format_error_messages() {
        let symbol_error = StockDataError::SymbolNotFound("Invalid symbol".to_string());
        let rate_limit_error = StockDataError::RateLimitExceeded;
        let network_error = StockDataError::NetworkError("Connection failed".to_string());

        assert!(format_stock_error(&symbol_error, Some("INVALID")).contains("❌ Stock symbol not found"));
        assert!(format_stock_error(&rate_limit_error, None).contains("⚠️ Rate limit exceeded"));
        assert!(format_stock_error(&network_error, None).contains("🌐 Network error"));
    }
}