/// Stock market data functionality
pub mod provider;

// Re-export commonly used types
pub use provider::{
    ProviderConfig, ProviderFactory, Sentiment, StockDataError, StockDataProvider, StockNews,
    StockQuote,
};

/// Initialize the stock module
pub fn init() {
    log::info!("Stock module initialized");
}