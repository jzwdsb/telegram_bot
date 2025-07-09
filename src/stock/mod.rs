/// Stock market data functionality
pub mod provider;
pub mod alpha_vantage;
pub mod database;
pub mod dynamodb;
pub mod service;

// Re-export commonly used types
pub use service::{StockService, format_stock_quote, format_stock_error};

