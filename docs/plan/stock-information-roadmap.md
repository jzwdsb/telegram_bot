# Stock Information Feature Roadmap

## Overview

This document outlines the development plan for adding daily stock information updates to the Telegram bot. The feature will allow groups to subscribe to stocks and receive AI-summarized daily updates about price changes and relevant news.

## Core Requirements

- **Data Source**: Alpha Vantage API (configurable)
- **Subscription Model**: Per-group subscriptions via `/subscribe` command
- **Daily Updates**: Scheduled at 10am UTC+8 (configurable per group)
- **Content**: Price changes and AI-summarized news impact
- **Storage**: DynamoDB for subscription management
- **AI Backend**: OpenAI (using bot's current model configuration)

## Development Phases

### Phase 1: Foundation & Architecture (Week 1-2)

#### 1.1 Data Source Abstraction Layer

- [ ] Design trait-based abstraction for stock data providers
  - `StockDataProvider` trait with methods for fetching quotes and news
  - Support for multiple data sources (Alpha Vantage first)
- [ ] Implement Alpha Vantage provider
  - Quote data fetching
  - News data fetching
  - Rate limiting and error handling
- [ ] Configuration management
  - Add `ALPHA_VANTAGE_API_KEY` to environment variables
  - Update Terraform variables for API key management

#### 1.2 Database Schema Design

- [ ] Design DynamoDB tables for stock subscriptions
  - Table: `stock-subscriptions`
    - Partition Key: `group_id`
    - Attributes: `stocks[]`, `schedule_time`, `timezone`, `enabled`
  - Table: `stock-schedule-index`
    - Partition Key: `schedule_hour`
    - Sort Key: `group_id`
    - For efficient scheduled task queries
- [ ] Update Terraform configuration for new tables

#### 1.3 Subscription Management Module

- [ ] Create `subscription.rs` module
  - Add/remove stock subscriptions
  - List current subscriptions
  - Validate stock symbols
  - Enforce 10-stock limit (configurable)
- [ ] Implement `/subscribe` command
  - Syntax: `/subscribe AAPL` or `/subscribe AAPL,MSFT,GOOGL`
- [ ] Implement `/unsubscribe` command
- [ ] Implement `/subscriptions` command to list current subscriptions

### Phase 2: Scheduled Tasks & AI Integration (Week 3-4)

#### 2.1 Scheduled Task System

- [ ] Design scheduler for Lambda environment
  - Use EventBridge for scheduled triggers
  - Lambda function for processing scheduled updates
- [ ] Implement group schedule management
  - Default: 10am UTC+8
  - Per-group schedule configuration via `/schedule` command
- [ ] Batch processing for efficiency

#### 2.2 AI Summary Integration

- [ ] Design prompt templates for stock news summarization
  - Focus on price impact analysis
  - Single sentence summary format
  - Positive/negative sentiment classification
- [ ] Implement news aggregation and summarization
  - Fetch relevant news for subscribed stocks
  - Send to OpenAI for summarization
  - Format for Telegram message

#### 2.3 Message Formatting

- [ ] Design daily update message format

  ```
  📈 Daily Stock Update - [Date]

  AAPL: $150.25 (+2.3%)
  📰 Positive: Strong iPhone sales drive revenue growth

  MSFT: $380.10 (-0.5%)
  📰 Neutral: Cloud competition impacts margins slightly

  [Additional stocks...]
  ```

- [ ] Implement message builder with proper formatting

### Phase 3: Core Features & Testing (Week 5-6)

#### 3.1 Command Implementation

- [ ] `/subscribe {symbol}` - Subscribe to stock
- [ ] `/unsubscribe {symbol}` - Unsubscribe from stock
- [ ] `/subscriptions` - List current subscriptions
- [ ] `/schedule {time}` - Set daily update time
- [ ] `/stockinfo {symbol}` - Get immediate stock info

#### 3.2 Integration Testing

- [ ] Test Alpha Vantage integration
- [ ] Test DynamoDB operations
- [ ] Test scheduled task execution
- [ ] Test AI summarization quality
- [ ] Test message delivery

#### 3.3 Documentation

- [ ] Update README with stock feature documentation
- [ ] Create user guide for stock commands
- [ ] Document configuration options

### Phase 4: Enhanced Features - High Priority (Week 7-8)

#### 4.1 Market News Aggregation

- [ ] Implement general market news feed
- [ ] Add `/marketnews` command
- [ ] AI summarization of market trends
- [ ] Include in daily updates optionally

#### 4.2 Technical Analysis Charts

- [ ] Integrate chart generation library
- [ ] Implement `/chart {symbol}` command
- [ ] Generate price charts with technical indicators
- [ ] Include charts in daily updates (optional)

### Phase 5: Enhanced Features - Medium Priority (Week 9-10)

#### 5.1 Real-time Price Alerts

- [ ] Design alert system architecture
- [ ] Implement `/alert {symbol} {condition}` command
  - Example: `/alert AAPL >150` or `/alert MSFT <-5%`
- [ ] Real-time monitoring system
- [ ] Instant notifications on trigger

#### 5.2 Portfolio Tracking

- [ ] Design portfolio data model
- [ ] Implement `/portfolio add {symbol} {quantity} {price}` command
- [ ] Track portfolio performance
- [ ] Include portfolio summary in daily updates

### Phase 6: Production Deployment (Week 11-12)

#### 6.1 Performance Optimization

- [ ] Optimize API calls and caching
- [ ] Implement rate limiting
- [ ] Optimize DynamoDB queries
- [ ] Lambda cold start optimization

#### 6.2 Monitoring & Observability

- [ ] Add CloudWatch metrics
- [ ] Set up alerts for failures
- [ ] Implement logging for debugging
- [ ] Create operational dashboards

#### 6.3 Production Deployment

- [ ] Deploy to production environment
- [ ] Monitor initial usage
- [ ] Gather user feedback
- [ ] Iterate based on feedback

## Configuration Requirements

### Environment Variables

```bash
# Stock Data Provider
STOCK_DATA_PROVIDER=alpha_vantage
ALPHA_VANTAGE_API_KEY=your_api_key_here

# Subscription Limits
MAX_STOCKS_PER_GROUP=10

# Default Schedule
DEFAULT_UPDATE_TIME=10:00
DEFAULT_TIMEZONE=UTC+8
```

### Terraform Variables

```hcl
variable "alpha_vantage_api_key" {
  description = "API key for Alpha Vantage stock data"
  type        = string
  sensitive   = true
}

variable "max_stocks_per_group" {
  description = "Maximum number of stocks a group can subscribe to"
  type        = number
  default     = 10
}

variable "default_update_schedule" {
  description = "Default time for daily stock updates"
  type        = string
  default     = "cron(0 2 * * ? *)" # 10am UTC+8
}
```

## Technical Architecture

### Module Structure

```
src/
├── stock/
│   ├── mod.rs              # Module exports
│   ├── provider.rs         # Data provider abstraction
│   ├── alpha_vantage.rs    # Alpha Vantage implementation
│   ├── subscription.rs     # Subscription management
│   ├── scheduler.rs        # Task scheduling
│   ├── summarizer.rs       # AI summarization
│   └── formatter.rs        # Message formatting
```

### Data Flow

1. EventBridge triggers Lambda at scheduled times
2. Lambda queries DynamoDB for groups scheduled at current time
3. For each group, fetch subscribed stock data from Alpha Vantage
4. Send news to OpenAI for summarization
5. Format and send message to Telegram group
6. Log results to CloudWatch

## Success Metrics

- Successfully deliver daily updates to 95%+ of scheduled groups
- AI summaries accurately reflect news sentiment 90%+ of the time
- Response time for commands < 2 seconds
- System uptime > 99.9%

## Risk Mitigation

- **API Rate Limits**: Implement caching and request batching
- **API Downtime**: Implement fallback providers and graceful degradation
- **Cost Management**: Monitor API usage and implement cost alerts
- **Data Accuracy**: Implement data validation and anomaly detection

## Future Enhancements

- Support for cryptocurrency data
- Integration with more data providers
- Advanced technical analysis
- Multi-language support for summaries
- Web dashboard for portfolio visualization
