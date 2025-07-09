/// Stock market data functionality
pub mod provider;
pub mod alpha_vantage;
pub mod database;
pub mod dynamodb;
pub mod service;

// Re-export commonly used types
pub use provider::{
    ProviderConfig, ProviderFactory, Sentiment, StockDataError, StockDataProvider, StockNews,
    StockQuote,
};
pub use alpha_vantage::AlphaVantageProvider;
pub use database::{
    DatabaseError, GroupConfig, NotificationLog, StockCache, StockDatabase, StockSubscription,
    SubscriptionSettings, UserPreferences,
};
pub use dynamodb::DynamoDbStockDatabase;
pub use service::{StockService, format_stock_quote, format_stock_error};

/// Initialize the stock module
pub fn init() {
    log::info!("Stock module initialized");
}