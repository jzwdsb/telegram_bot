use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DynamoDB table structure for group stock subscriptions
/// Table Name: telegram_bot_stock_subscriptions
/// Primary Key: group_id (String)
/// Sort Key: stock_symbol (String)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSubscription {
    /// Telegram group/chat ID (Primary Key)
    pub group_id: String,
    
    /// Stock symbol (Sort Key) - e.g., "AAPL", "TSLA"
    pub stock_symbol: String,
    
    /// When this subscription was created
    pub created_at: DateTime<Utc>,
    
    /// When this subscription was last updated
    pub updated_at: DateTime<Utc>,
    
    /// Whether this subscription is active
    pub is_active: bool,
    
    /// User ID who created the subscription
    pub created_by_user_id: i64,
    
    /// Optional custom settings for this subscription
    pub settings: Option<SubscriptionSettings>,
}

/// Settings for individual stock subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionSettings {
    /// Custom notification time (if different from group default)
    pub notification_time: Option<String>, // Format: "HH:MM" in UTC+8
    
    /// Whether to include AI summary for this stock
    pub include_ai_summary: bool,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// DynamoDB table structure for group configuration
/// Table Name: telegram_bot_group_config
/// Primary Key: group_id (String)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    /// Telegram group/chat ID (Primary Key)
    pub group_id: String,
    
    /// Group title/name for reference
    pub group_title: Option<String>,
    
    /// Maximum number of stock subscriptions allowed (default: 10)
    pub max_subscriptions: u32,
    
    /// Default notification time in UTC+8 (default: "10:00")
    pub default_notification_time: String,
    
    /// Timezone for this group (default: "Asia/Shanghai")
    pub timezone: String,
    
    /// Whether AI summaries are enabled for this group
    pub ai_summaries_enabled: bool,
    
    /// List of admin user IDs who can manage subscriptions
    pub admin_user_ids: Vec<i64>,
    
    /// When this group was first configured
    pub created_at: DateTime<Utc>,
    
    /// When this group config was last updated
    pub updated_at: DateTime<Utc>,
    
    /// Whether this group is active
    pub is_active: bool,
    
    /// Additional group settings
    pub settings: HashMap<String, String>,
}

/// DynamoDB table structure for user preferences
/// Table Name: telegram_bot_user_preferences  
/// Primary Key: user_id (Number)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Telegram user ID (Primary Key)
    pub user_id: i64,
    
    /// User's display name for reference
    pub username: Option<String>,
    
    /// User's preferred timezone (default: "Asia/Shanghai")
    pub timezone: String,
    
    /// Whether user wants to receive private notifications
    pub private_notifications_enabled: bool,
    
    /// User's preferred AI model for summaries
    pub preferred_ai_model: Option<String>,
    
    /// When this user was first seen
    pub created_at: DateTime<Utc>,
    
    /// When preferences were last updated
    pub updated_at: DateTime<Utc>,
    
    /// Additional user settings
    pub settings: HashMap<String, String>,
}

/// DynamoDB table structure for stock data cache
/// Table Name: telegram_bot_stock_cache
/// Primary Key: stock_symbol (String)
/// TTL: expires_at (Number) - Auto-cleanup after 24 hours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockCache {
    /// Stock symbol (Primary Key)
    pub stock_symbol: String,
    
    /// Cached stock quote data (JSON string)
    pub quote_data: String,
    
    /// Cached news data (JSON string) 
    pub news_data: String,
    
    /// When this cache entry was created
    pub cached_at: DateTime<Utc>,
    
    /// TTL timestamp for auto-cleanup (Unix timestamp)
    pub expires_at: i64,
    
    /// Data source provider name
    pub provider: String,
    
    /// Version for cache invalidation
    pub cache_version: u32,
}

/// DynamoDB table structure for notification logs
/// Table Name: telegram_bot_notification_logs
/// Primary Key: log_id (String) - UUID
/// GSI: group_id-timestamp-index for querying by group and time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationLog {
    /// Unique log ID (Primary Key) - UUID
    pub log_id: String,
    
    /// Group ID this notification was sent to
    pub group_id: String,
    
    /// Stock symbol this notification was about
    pub stock_symbol: String,
    
    /// When the notification was sent
    pub timestamp: DateTime<Utc>,
    
    /// Whether the notification was sent successfully
    pub success: bool,
    
    /// Error message if failed
    pub error_message: Option<String>,
    
    /// Type of notification (daily_update, alert, etc.)
    pub notification_type: String,
    
    /// Message content that was sent
    pub message_content: String,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    
    /// TTL for auto-cleanup after 30 days
    pub expires_at: i64,
}

/// Database operations trait for stock subscription management
#[async_trait::async_trait]
pub trait StockDatabase: Send + Sync {
    // Subscription management
    async fn create_subscription(&self, subscription: StockSubscription) -> Result<(), DatabaseError>;
    async fn get_subscription(&self, group_id: &str, stock_symbol: &str) -> Result<Option<StockSubscription>, DatabaseError>;
    async fn list_subscriptions(&self, group_id: &str) -> Result<Vec<StockSubscription>, DatabaseError>;
    async fn update_subscription(&self, subscription: StockSubscription) -> Result<(), DatabaseError>;
    async fn delete_subscription(&self, group_id: &str, stock_symbol: &str) -> Result<(), DatabaseError>;
    async fn count_subscriptions(&self, group_id: &str) -> Result<u32, DatabaseError>;
    
    // Group configuration
    async fn create_group_config(&self, config: GroupConfig) -> Result<(), DatabaseError>;
    async fn get_group_config(&self, group_id: &str) -> Result<Option<GroupConfig>, DatabaseError>;
    async fn update_group_config(&self, config: GroupConfig) -> Result<(), DatabaseError>;
    async fn list_active_groups(&self) -> Result<Vec<GroupConfig>, DatabaseError>;
    
    // User preferences
    async fn create_user_preferences(&self, preferences: UserPreferences) -> Result<(), DatabaseError>;
    async fn get_user_preferences(&self, user_id: i64) -> Result<Option<UserPreferences>, DatabaseError>;
    async fn update_user_preferences(&self, preferences: UserPreferences) -> Result<(), DatabaseError>;
    
    // Cache management
    async fn set_cache(&self, cache: StockCache) -> Result<(), DatabaseError>;
    async fn get_cache(&self, stock_symbol: &str) -> Result<Option<StockCache>, DatabaseError>;
    async fn invalidate_cache(&self, stock_symbol: &str) -> Result<(), DatabaseError>;
    
    // Notification logging
    async fn log_notification(&self, log: NotificationLog) -> Result<(), DatabaseError>;
    async fn get_recent_notifications(&self, group_id: &str, hours: u32) -> Result<Vec<NotificationLog>, DatabaseError>;
    
    // Health check
    async fn health_check(&self) -> Result<(), DatabaseError>;
}

/// Database error types
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Item not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Conflict error: {0}")]
    ConflictError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Helper functions for database operations
impl StockSubscription {
    /// Create a new subscription
    pub fn new(
        group_id: String,
        stock_symbol: String,
        created_by_user_id: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            group_id,
            stock_symbol: stock_symbol.to_uppercase(),
            created_at: now,
            updated_at: now,
            is_active: true,
            created_by_user_id,
            settings: None,
        }
    }
    
    /// Update the subscription's timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl GroupConfig {
    /// Create a new group configuration with defaults
    pub fn new(group_id: String, admin_user_id: i64) -> Self {
        let now = Utc::now();
        Self {
            group_id,
            group_title: None,
            max_subscriptions: 10,
            default_notification_time: "10:00".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            ai_summaries_enabled: true,
            admin_user_ids: vec![admin_user_id],
            created_at: now,
            updated_at: now,
            is_active: true,
            settings: HashMap::new(),
        }
    }
    
    /// Check if user is an admin for this group
    pub fn is_admin(&self, user_id: i64) -> bool {
        self.admin_user_ids.contains(&user_id)
    }
    
    /// Add admin user
    pub fn add_admin(&mut self, user_id: i64) {
        if !self.admin_user_ids.contains(&user_id) {
            self.admin_user_ids.push(user_id);
            self.touch();
        }
    }
    
    /// Remove admin user
    pub fn remove_admin(&mut self, user_id: i64) {
        self.admin_user_ids.retain(|&id| id != user_id);
        self.touch();
    }
    
    /// Update the config's timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl UserPreferences {
    /// Create new user preferences with defaults
    pub fn new(user_id: i64, username: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            username,
            timezone: "Asia/Shanghai".to_string(),
            private_notifications_enabled: false,
            preferred_ai_model: None,
            created_at: now,
            updated_at: now,
            settings: HashMap::new(),
        }
    }
    
    /// Update the preferences timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl StockCache {
    /// Create new cache entry
    pub fn new(
        stock_symbol: String,
        quote_data: String,
        news_data: String,
        provider: String,
        ttl_hours: u32,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now.timestamp() + (ttl_hours as i64 * 3600);
        
        Self {
            stock_symbol: stock_symbol.to_uppercase(),
            quote_data,
            news_data,
            cached_at: now,
            expires_at,
            provider,
            cache_version: 1,
        }
    }
    
    /// Check if cache entry is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() >= self.expires_at
    }
}

impl NotificationLog {
    /// Create new notification log entry
    pub fn new(
        group_id: String,
        stock_symbol: String,
        notification_type: String,
        message_content: String,
        processing_time_ms: u64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now.timestamp() + (30 * 24 * 3600); // 30 days TTL
        
        Self {
            log_id: uuid::Uuid::new_v4().to_string(),
            group_id,
            stock_symbol: stock_symbol.to_uppercase(),
            timestamp: now,
            success: true,
            error_message: None,
            notification_type,
            message_content,
            processing_time_ms,
            expires_at,
        }
    }
    
    /// Mark log as failed with error message
    pub fn with_error(mut self, error_message: String) -> Self {
        self.success = false;
        self.error_message = Some(error_message);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_subscription_creation() {
        let subscription = StockSubscription::new(
            "-1001234567890".to_string(),
            "aapl".to_string(),
            123456789,
        );
        
        assert_eq!(subscription.group_id, "-1001234567890");
        assert_eq!(subscription.stock_symbol, "AAPL"); // Should be uppercase
        assert_eq!(subscription.created_by_user_id, 123456789);
        assert!(subscription.is_active);
        assert!(subscription.settings.is_none());
    }

    #[test]
    fn test_group_config_admin_management() {
        let mut config = GroupConfig::new("-1001234567890".to_string(), 123456789);
        
        assert!(config.is_admin(123456789));
        assert!(!config.is_admin(987654321));
        
        config.add_admin(987654321);
        assert!(config.is_admin(987654321));
        assert_eq!(config.admin_user_ids.len(), 2);
        
        config.remove_admin(123456789);
        assert!(!config.is_admin(123456789));
        assert_eq!(config.admin_user_ids.len(), 1);
    }

    #[test]
    fn test_stock_cache_expiration() {
        let cache = StockCache::new(
            "AAPL".to_string(),
            "{}".to_string(),
            "[]".to_string(),
            "alpha_vantage".to_string(),
            24,
        );
        
        assert!(!cache.is_expired());
        assert_eq!(cache.stock_symbol, "AAPL");
        assert_eq!(cache.provider, "alpha_vantage");
    }

    #[test]
    fn test_notification_log_creation() {
        let log = NotificationLog::new(
            "-1001234567890".to_string(),
            "AAPL".to_string(),
            "daily_update".to_string(),
            "AAPL: $150.00 (+1.25%)".to_string(),
            500,
        );
        
        assert_eq!(log.group_id, "-1001234567890");
        assert_eq!(log.stock_symbol, "AAPL");
        assert!(log.success);
        assert!(log.error_message.is_none());
        assert!(!log.log_id.is_empty());
    }
}