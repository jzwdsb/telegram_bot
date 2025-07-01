# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Telegram bot implementation built with Rust using the teloxide framework. The bot features automatic environment detection to switch between polling mode (development) and webhook mode (production), with an async architecture using tokio runtime and command-based interaction pattern.

## Core Architecture

The bot follows a dual-mode architecture with automatic environment detection:

- **Command System**: Commands defined as enum using teloxide's `BotCommands` derive macro
- **Handler Pattern**: Each command processed by the `answer` function with pattern matching
- **Environment Detection**: Automatic mode switching via `is_production_environment()` function
- **Dual Runtime Modes**:
  - **Polling Mode**: Uses teloxide's REPL for local development
  - **Webhook Mode**: Uses axum web server with webhook endpoints for production
- **Configuration**: Environment-based via `TELOXIDE_TOKEN` with dotenv support
- **AI Integration**: Extensible AI backend system supporting multiple AI providers

## Available Commands

- `/help` - Display available commands and descriptions
- `/username <name>` - Handle username input
- `/usernameandage <name> <age>` - Handle username and age input
- `/general <message>` - Chat with AI (requires OPENAI_API_KEY)

### Group Chat Usage

In group chats, mention the bot with `@botname` followed by your command or message:

- `@yourbotname /help` - Get help in group chat
- `@yourbotname /username John` - Commands with parameters
- `@yourbotname Hello, how are you?` - Direct AI chat (no /general needed)
- `@yourbotname` - Just mentioning shows available commands

In private chats, commands work normally without needing to mention the bot:

- `/help` - Get help
- `/general <message>` - Chat with AI
- Or just send any message for AI chat

## AI Backend System

The bot features an extensible AI backend architecture:

- **Trait-based Design**: `AiBackend` trait allows multiple AI providers
- **Current Support**: OpenAI ChatGPT (gpt-3.5-turbo)
- **Future Extensible**: Easy to add support for other AI services
- **Error Handling**: Graceful fallback and user-friendly error messages
- **Configuration**: Environment variable based setup

## Development Commands

### Local Development

```bash
# Run in polling mode (auto-detected)
cargo run

# With debug logging
RUST_LOG=debug cargo run
```

### Production Build

```bash
cargo build --release
```

### Environment Setup

```bash
cp .env.example .env
# Edit .env and set TELOXIDE_TOKEN=your_actual_bot_token
```

## Environment Detection Logic

The bot automatically detects production environments by checking for:

- Cloud platform indicators (`RAILWAY_ENVIRONMENT`, `HEROKU_APP_NAME`, `VERCEL`)
- Environment variables (`NODE_ENV=production`, `ENVIRONMENT=production`)
- Production configuration (`PORT` + `WEBHOOK_URL` both present)
- Manual override (`WEBHOOK_MODE=true`)

**Local Development** → Polling Mode (no extra setup)
**Production Deployment** → Webhook Mode (requires `WEBHOOK_URL`)

## Telegram Bot Configuration

### BotFather Settings (Required for Group Chats)

To make your bot work in group chats, configure these settings with @BotFather:

1. **Privacy Mode**: `/setprivacy` → `Disable`

   - This allows the bot to see all messages in groups
   - Default `Enable` only lets bot see messages that start with `/` or mention the bot

2. **Group Admin**: Add your bot as an admin in the group (optional but recommended)

3. **Commands Menu**: `/setcommands` with:
   ```
   help - Display available commands and descriptions
   username - Handle username input
   usernameandage - Handle username and age input
   general - Chat with AI - send your message after the command
   ```

### Group Chat Behavior

- **Natural Mentions**: Use `@yourbotname /command` or `@yourbotname message`
- **AI Chat**: Any message after `@yourbotname` becomes an AI conversation
- **Privacy Mode Disabled**: Bot sees all messages but only responds when mentioned
- **Privacy Mode Enabled**: Bot only sees `/commands` and `@mentions` (recommended setting)

## Production Deployment

The bot supports multiple deployment modes with automatic environment detection:

### AWS Lambda Deployment (Recommended)

**Build for Lambda:**

```bash
cargo build --release --features lambda --no-default-features
```

**Required Environment Variables:**

```bash
TELOXIDE_TOKEN=your_bot_token
OPENAI_API_KEY=your_openai_api_key  # Required for AI chat
WEBHOOK_URL=https://your-lambda-url.amazonaws.com/webhook  # Lambda Function URL
AWS_LAMBDA_FUNCTION_NAME=telegram_bot  # Auto-set by Lambda
```

**Lambda Configuration:**

- Runtime: Custom Runtime (use `cargo-lambda` or build manually)
- Handler: `bootstrap` (for Rust Lambda)
- Timeout: 30 seconds minimum
- Memory: 256MB minimum
- Function URL: Enable with webhook endpoint

### Traditional Server Deployment

**Build for Axum server:**

```bash
cargo build --release  # Uses default axum-server feature
```

**Required Environment Variables:**

```bash
TELOXIDE_TOKEN=your_bot_token
OPENAI_API_KEY=your_openai_api_key  # Required for AI chat
WEBHOOK_URL=https://your-domain.com/webhook
PORT=8080  # Usually auto-provided by cloud platforms
```

**Webhook Endpoints:**

- `GET /` - Health check endpoint
- `POST /webhook` - Telegram webhook handler

### Local Development

**Run in polling mode:**

```bash
RUST_LOG=info cargo run  # Automatically uses polling mode
```

## AWS Lambda Deployment

The bot includes complete AWS infrastructure setup using Terraform:

### Quick Deployment

```bash
# 1. Configure deployment
cp deployment/terraform/terraform.tfvars.example deployment/terraform/terraform.tfvars
# Edit terraform.tfvars with your Telegram token and settings

# 2. Deploy everything
./deployment/deploy.sh
```

### Prerequisites

- `cargo install cargo-lambda` - For building Lambda functions
- `terraform` - Infrastructure as Code
- `aws configure` - AWS credentials setup

### Infrastructure Includes

- **AWS Lambda Function** with public Function URL
- **IAM roles** with minimal permissions
- **CloudWatch logging** with configurable retention
- **Automatic webhook** configuration

See `deployment/README.md` for detailed deployment instructions.

## Key Dependencies

- `teloxide`: Main bot framework with "macros" and "webhooks" features
- `axum`: Web server for webhook mode
- `tokio`: Async runtime with multi-thread and macros features
- `dotenvy`: Environment file loading
- `log` + `pretty_env_logger`: Logging infrastructure
- `reqwest`: HTTP client for AI backend API calls
- `serde`: JSON serialization/deserialization
- `async-trait`: Async trait support for extensible AI backends
- Standard Rust 2024 edition

## Configuration Management

The bot uses a layered configuration approach:

1. `.env` file loading via dotenvy (development)
2. System environment variables (production)
3. Automatic environment detection for mode switching
4. Manual override capabilities via `WEBHOOK_MODE`
