use super::database::{
    DatabaseError, GroupConfig, NotificationLog, StockCache, StockDatabase, StockSubscription,
    UserPreferences,
};
use aws_sdk_dynamodb::{
    error::SdkError,
    types::AttributeValue,
    Client as DynamoClient,
};
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;

/// DynamoDB implementation of StockDatabase trait
pub struct DynamoDbStockDatabase {
    client: DynamoClient,
    table_prefix: String,
}

impl DynamoDbStockDatabase {
    /// Create a new DynamoDB stock database instance
    pub fn new(client: DynamoClient, table_prefix: String) -> Self {
        Self {
            client,
            table_prefix,
        }
    }

    /// Get table name with prefix
    fn table_name(&self, base_name: &str) -> String {
        format!("{}_{}", self.table_prefix, base_name)
    }

    /// Convert StockSubscription to DynamoDB item
    fn subscription_to_item(&self, subscription: &StockSubscription) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        
        item.insert("group_id".to_string(), AttributeValue::S(subscription.group_id.clone()));
        item.insert("stock_symbol".to_string(), AttributeValue::S(subscription.stock_symbol.clone()));
        item.insert("created_at".to_string(), AttributeValue::S(subscription.created_at.to_rfc3339()));
        item.insert("updated_at".to_string(), AttributeValue::S(subscription.updated_at.to_rfc3339()));
        item.insert("is_active".to_string(), AttributeValue::Bool(subscription.is_active));
        item.insert("created_by_user_id".to_string(), AttributeValue::N(subscription.created_by_user_id.to_string()));
        
        if let Some(settings) = &subscription.settings {
            if let Ok(settings_json) = serde_json::to_string(settings) {
                item.insert("settings".to_string(), AttributeValue::S(settings_json));
            }
        }
        
        item
    }

    /// Convert DynamoDB item to StockSubscription
    fn item_to_subscription(&self, item: HashMap<String, AttributeValue>) -> Result<StockSubscription, DatabaseError> {
        let group_id = item.get("group_id")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| DatabaseError::SerializationError("Missing group_id".to_string()))?
            .clone();

        let stock_symbol = item.get("stock_symbol")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| DatabaseError::SerializationError("Missing stock_symbol".to_string()))?
            .clone();

        let created_at = item.get("created_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| DatabaseError::SerializationError("Invalid created_at".to_string()))?;

        let updated_at = item.get("updated_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| DatabaseError::SerializationError("Invalid updated_at".to_string()))?;

        let is_active = item.get("is_active")
            .and_then(|v| v.as_bool().ok())
            .copied()
            .unwrap_or(true);

        let created_by_user_id = item.get("created_by_user_id")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<i64>().ok())
            .ok_or_else(|| DatabaseError::SerializationError("Invalid created_by_user_id".to_string()))?;

        let settings = item.get("settings")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| serde_json::from_str(s).ok());

        Ok(StockSubscription {
            group_id,
            stock_symbol,
            created_at,
            updated_at,
            is_active,
            created_by_user_id,
            settings,
        })
    }

    /// Convert GroupConfig to DynamoDB item
    fn group_config_to_item(&self, config: &GroupConfig) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        
        item.insert("group_id".to_string(), AttributeValue::S(config.group_id.clone()));
        
        if let Some(title) = &config.group_title {
            item.insert("group_title".to_string(), AttributeValue::S(title.clone()));
        }
        
        item.insert("max_subscriptions".to_string(), AttributeValue::N(config.max_subscriptions.to_string()));
        item.insert("default_notification_time".to_string(), AttributeValue::S(config.default_notification_time.clone()));
        item.insert("timezone".to_string(), AttributeValue::S(config.timezone.clone()));
        item.insert("ai_summaries_enabled".to_string(), AttributeValue::Bool(config.ai_summaries_enabled));
        item.insert("created_at".to_string(), AttributeValue::S(config.created_at.to_rfc3339()));
        item.insert("updated_at".to_string(), AttributeValue::S(config.updated_at.to_rfc3339()));
        item.insert("is_active".to_string(), AttributeValue::Bool(config.is_active));
        
        // Convert admin_user_ids to string list
        let admin_ids: Vec<AttributeValue> = config.admin_user_ids
            .iter()
            .map(|id| AttributeValue::N(id.to_string()))
            .collect();
        item.insert("admin_user_ids".to_string(), AttributeValue::L(admin_ids));
        
        // Convert settings map
        if !config.settings.is_empty() {
            if let Ok(settings_json) = serde_json::to_string(&config.settings) {
                item.insert("settings".to_string(), AttributeValue::S(settings_json));
            }
        }
        
        item
    }

    /// Convert DynamoDB item to GroupConfig
    fn item_to_group_config(&self, item: HashMap<String, AttributeValue>) -> Result<GroupConfig, DatabaseError> {
        let group_id = item.get("group_id")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| DatabaseError::SerializationError("Missing group_id".to_string()))?
            .clone();

        let group_title = item.get("group_title")
            .and_then(|v| v.as_s().ok())
            .map(|s| s.clone());

        let max_subscriptions = item.get("max_subscriptions")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(10);

        let default_notification_time = item.get("default_notification_time")
            .and_then(|v| v.as_s().ok())
            .unwrap_or(&"10:00".to_string())
            .clone();

        let timezone = item.get("timezone")
            .and_then(|v| v.as_s().ok())
            .unwrap_or(&"Asia/Shanghai".to_string())
            .clone();

        let ai_summaries_enabled = item.get("ai_summaries_enabled")
            .and_then(|v| v.as_bool().ok())
            .copied()
            .unwrap_or(true);

        let created_at = item.get("created_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| DatabaseError::SerializationError("Invalid created_at".to_string()))?;

        let updated_at = item.get("updated_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| DatabaseError::SerializationError("Invalid updated_at".to_string()))?;

        let is_active = item.get("is_active")
            .and_then(|v| v.as_bool().ok())
            .copied()
            .unwrap_or(true);

        // Parse admin_user_ids
        let admin_user_ids = item.get("admin_user_ids")
            .and_then(|v| v.as_l().ok())
            .map(|list| {
                list.iter()
                    .filter_map(|v| v.as_n().ok())
                    .filter_map(|n| n.parse::<i64>().ok())
                    .collect()
            })
            .unwrap_or_default();

        // Parse settings
        let settings = item.get("settings")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        Ok(GroupConfig {
            group_id,
            group_title,
            max_subscriptions,
            default_notification_time,
            timezone,
            ai_summaries_enabled,
            admin_user_ids,
            created_at,
            updated_at,
            is_active,
            settings,
        })
    }
}

#[async_trait::async_trait]
impl StockDatabase for DynamoDbStockDatabase {
    async fn create_subscription(&self, subscription: StockSubscription) -> Result<(), DatabaseError> {
        let table_name = self.table_name("stock_subscriptions");
        let item = self.subscription_to_item(&subscription);

        let result = self
            .client
            .put_item()
            .table_name(&table_name)
            .set_item(Some(item))
            .condition_expression("attribute_not_exists(#gid) AND attribute_not_exists(#ss)")
            .expression_attribute_names("#gid", "group_id")
            .expression_attribute_names("#ss", "stock_symbol")
            .send()
            .await;

        match result {
            Ok(_) => {
                log::info!("Created subscription for {} in group {}", subscription.stock_symbol, subscription.group_id);
                Ok(())
            }
            Err(SdkError::ServiceError(err)) if err.err().is_conditional_check_failed_exception() => {
                Err(DatabaseError::ConflictError(
                    "Subscription already exists".to_string(),
                ))
            }
            Err(e) => {
                log::error!("Failed to create subscription: {:?}", e);
                Err(DatabaseError::Unknown(format!("DynamoDB error: {:?}", e)))
            }
        }
    }

    async fn get_subscription(&self, group_id: &str, stock_symbol: &str) -> Result<Option<StockSubscription>, DatabaseError> {
        let table_name = self.table_name("stock_subscriptions");

        let result = self
            .client
            .get_item()
            .table_name(&table_name)
            .key("group_id", AttributeValue::S(group_id.to_string()))
            .key("stock_symbol", AttributeValue::S(stock_symbol.to_uppercase()))
            .send()
            .await;

        match result {
            Ok(output) => {
                if let Some(item) = output.item {
                    let subscription = self.item_to_subscription(item)?;
                    Ok(Some(subscription))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                log::error!("Failed to get subscription: {:?}", e);
                Err(DatabaseError::Unknown(format!("DynamoDB error: {:?}", e)))
            }
        }
    }

    async fn list_subscriptions(&self, group_id: &str) -> Result<Vec<StockSubscription>, DatabaseError> {
        let table_name = self.table_name("stock_subscriptions");

        let result = self
            .client
            .query()
            .table_name(&table_name)
            .key_condition_expression("group_id = :gid")
            .filter_expression("is_active = :active")
            .expression_attribute_values(":gid", AttributeValue::S(group_id.to_string()))
            .expression_attribute_values(":active", AttributeValue::Bool(true))
            .send()
            .await;

        match result {
            Ok(output) => {
                let mut subscriptions = Vec::new();
                if let Some(items) = output.items {
                    for item in items {
                        match self.item_to_subscription(item) {
                            Ok(subscription) => subscriptions.push(subscription),
                            Err(e) => log::warn!("Failed to parse subscription: {:?}", e),
                        }
                    }
                }
                Ok(subscriptions)
            }
            Err(e) => {
                log::error!("Failed to list subscriptions: {:?}", e);
                Err(DatabaseError::Unknown(format!("DynamoDB error: {:?}", e)))
            }
        }
    }

    async fn update_subscription(&self, subscription: StockSubscription) -> Result<(), DatabaseError> {
        let table_name = self.table_name("stock_subscriptions");
        let item = self.subscription_to_item(&subscription);

        let result = self
            .client
            .put_item()
            .table_name(&table_name)
            .set_item(Some(item))
            .send()
            .await;

        match result {
            Ok(_) => {
                log::info!("Updated subscription for {} in group {}", subscription.stock_symbol, subscription.group_id);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to update subscription: {:?}", e);
                Err(DatabaseError::Unknown(format!("DynamoDB error: {:?}", e)))
            }
        }
    }

    async fn delete_subscription(&self, group_id: &str, stock_symbol: &str) -> Result<(), DatabaseError> {
        let table_name = self.table_name("stock_subscriptions");

        let result = self
            .client
            .delete_item()
            .table_name(&table_name)
            .key("group_id", AttributeValue::S(group_id.to_string()))
            .key("stock_symbol", AttributeValue::S(stock_symbol.to_uppercase()))
            .send()
            .await;

        match result {
            Ok(_) => {
                log::info!("Deleted subscription for {} in group {}", stock_symbol, group_id);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to delete subscription: {:?}", e);
                Err(DatabaseError::Unknown(format!("DynamoDB error: {:?}", e)))
            }
        }
    }

    async fn count_subscriptions(&self, group_id: &str) -> Result<u32, DatabaseError> {
        let table_name = self.table_name("stock_subscriptions");

        let result = self
            .client
            .query()
            .table_name(&table_name)
            .key_condition_expression("group_id = :gid")
            .filter_expression("is_active = :active")
            .expression_attribute_values(":gid", AttributeValue::S(group_id.to_string()))
            .expression_attribute_values(":active", AttributeValue::Bool(true))
            .select(aws_sdk_dynamodb::types::Select::Count)
            .send()
            .await;

        match result {
            Ok(output) => Ok(output.count() as u32),
            Err(e) => {
                log::error!("Failed to count subscriptions: {:?}", e);
                Err(DatabaseError::Unknown(format!("DynamoDB error: {:?}", e)))
            }
        }
    }

    async fn create_group_config(&self, config: GroupConfig) -> Result<(), DatabaseError> {
        let table_name = self.table_name("group_config");
        let item = self.group_config_to_item(&config);

        let result = self
            .client
            .put_item()
            .table_name(&table_name)
            .set_item(Some(item))
            .condition_expression("attribute_not_exists(group_id)")
            .send()
            .await;

        match result {
            Ok(_) => {
                log::info!("Created group config for {}", config.group_id);
                Ok(())
            }
            Err(SdkError::ServiceError(err)) if err.err().is_conditional_check_failed_exception() => {
                Err(DatabaseError::ConflictError(
                    "Group config already exists".to_string(),
                ))
            }
            Err(e) => {
                log::error!("Failed to create group config: {:?}", e);
                Err(DatabaseError::Unknown(format!("DynamoDB error: {:?}", e)))
            }
        }
    }

    async fn get_group_config(&self, group_id: &str) -> Result<Option<GroupConfig>, DatabaseError> {
        let table_name = self.table_name("group_config");

        let result = self
            .client
            .get_item()
            .table_name(&table_name)
            .key("group_id", AttributeValue::S(group_id.to_string()))
            .send()
            .await;

        match result {
            Ok(output) => {
                if let Some(item) = output.item {
                    let config = self.item_to_group_config(item)?;
                    Ok(Some(config))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                log::error!("Failed to get group config: {:?}", e);
                Err(DatabaseError::Unknown(format!("DynamoDB error: {:?}", e)))
            }
        }
    }

    async fn update_group_config(&self, config: GroupConfig) -> Result<(), DatabaseError> {
        let table_name = self.table_name("group_config");
        let item = self.group_config_to_item(&config);

        let result = self
            .client
            .put_item()
            .table_name(&table_name)
            .set_item(Some(item))
            .send()
            .await;

        match result {
            Ok(_) => {
                log::info!("Updated group config for {}", config.group_id);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to update group config: {:?}", e);
                Err(DatabaseError::Unknown(format!("DynamoDB error: {:?}", e)))
            }
        }
    }

    async fn list_active_groups(&self) -> Result<Vec<GroupConfig>, DatabaseError> {
        let table_name = self.table_name("group_config");

        let result = self
            .client
            .scan()
            .table_name(&table_name)
            .filter_expression("is_active = :active")
            .expression_attribute_values(":active", AttributeValue::Bool(true))
            .send()
            .await;

        match result {
            Ok(output) => {
                let mut configs = Vec::new();
                if let Some(items) = output.items {
                    for item in items {
                        match self.item_to_group_config(item) {
                            Ok(config) => configs.push(config),
                            Err(e) => log::warn!("Failed to parse group config: {:?}", e),
                        }
                    }
                }
                Ok(configs)
            }
            Err(e) => {
                log::error!("Failed to list active groups: {:?}", e);
                Err(DatabaseError::Unknown(format!("DynamoDB error: {:?}", e)))
            }
        }
    }

    async fn create_user_preferences(&self, _preferences: UserPreferences) -> Result<(), DatabaseError> {
        // TODO: Implement user preferences operations
        log::warn!("User preferences operations not yet implemented");
        Ok(())
    }

    async fn get_user_preferences(&self, _user_id: i64) -> Result<Option<UserPreferences>, DatabaseError> {
        // TODO: Implement user preferences operations
        Ok(None)
    }

    async fn update_user_preferences(&self, _preferences: UserPreferences) -> Result<(), DatabaseError> {
        // TODO: Implement user preferences operations
        Ok(())
    }

    async fn set_cache(&self, _cache: StockCache) -> Result<(), DatabaseError> {
        // TODO: Implement cache operations
        log::warn!("Cache operations not yet implemented");
        Ok(())
    }

    async fn get_cache(&self, _stock_symbol: &str) -> Result<Option<StockCache>, DatabaseError> {
        // TODO: Implement cache operations
        Ok(None)
    }

    async fn invalidate_cache(&self, _stock_symbol: &str) -> Result<(), DatabaseError> {
        // TODO: Implement cache operations
        Ok(())
    }

    async fn log_notification(&self, _log: NotificationLog) -> Result<(), DatabaseError> {
        // TODO: Implement notification logging
        log::warn!("Notification logging not yet implemented");
        Ok(())
    }

    async fn get_recent_notifications(&self, _group_id: &str, _hours: u32) -> Result<Vec<NotificationLog>, DatabaseError> {
        // TODO: Implement notification logging
        Ok(Vec::new())
    }

    async fn health_check(&self) -> Result<(), DatabaseError> {
        // Simple health check by listing tables
        match self.client.list_tables().send().await {
            Ok(_) => {
                log::debug!("DynamoDB health check passed");
                Ok(())
            }
            Err(e) => {
                log::error!("DynamoDB health check failed: {:?}", e);
                Err(DatabaseError::ConnectionError(format!("Health check failed: {:?}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_subscription_to_item_conversion() {
        let subscription = StockSubscription::new(
            "-1001234567890".to_string(),
            "AAPL".to_string(),
            123456789,
        );

        let db = DynamoDbStockDatabase::new(
            // We can't create a real client in tests, but we can test the conversion logic
            aws_sdk_dynamodb::Client::from_conf_conn(
                aws_sdk_dynamodb::Config::builder().build(),
                aws_smithy_runtime::client::http::test_util::StaticReplayConnector::new(
                    Vec::new()
                )
            ),
            "test".to_string(),
        );

        let item = db.subscription_to_item(&subscription);
        
        assert!(item.contains_key("group_id"));
        assert!(item.contains_key("stock_symbol"));
        assert!(item.contains_key("created_at"));
        assert!(item.contains_key("is_active"));
        
        // Test round-trip conversion
        let converted_back = db.item_to_subscription(item).unwrap();
        assert_eq!(converted_back.group_id, subscription.group_id);
        assert_eq!(converted_back.stock_symbol, subscription.stock_symbol);
        assert_eq!(converted_back.created_by_user_id, subscription.created_by_user_id);
    }

    #[test]
    fn test_table_name_generation() {
        let db = DynamoDbStockDatabase::new(
            aws_sdk_dynamodb::Client::from_conf_conn(
                aws_sdk_dynamodb::Config::builder().build(),
                aws_smithy_runtime::client::http::test_util::StaticReplayConnector::new(
                    Vec::new()
                )
            ),
            "telegram_bot".to_string(),
        );

        assert_eq!(db.table_name("stock_subscriptions"), "telegram_bot_stock_subscriptions");
        assert_eq!(db.table_name("group_config"), "telegram_bot_group_config");
    }
}