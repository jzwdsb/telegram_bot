# 🤖 Telegram Bot in Rust

A production-ready Telegram bot implementation built with Rust using the teloxide framework. Features automatic environment detection, dual deployment modes, and extensible AI integration.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Deployment](https://img.shields.io/badge/deployment-AWS%20Lambda-yellow.svg)](https://aws.amazon.com/lambda/)

## ✨ Features

- 🚀 **Dual Deployment Modes**: Automatic switching between polling (development) and webhook (production)
- ☁️ **AWS Lambda Support**: Production-ready serverless deployment with Terraform
- 🤖 **AI Integration**: Extensible AI backend with OpenAI ChatGPT support
- 📈 **Stock Market Data**: Real-time stock quotes via Alpha Vantage integration
- 💬 **Smart Group Chat**: Natural @mention handling for group conversations
- 🔧 **Environment Detection**: Automatic mode switching based on deployment environment
- 📝 **Comprehensive Logging**: Detailed logging with emoji indicators for easy monitoring
- 🛡️ **Security Best Practices**: Proper credential management and validation

## 🏗️ Architecture

The bot follows a modular architecture with automatic environment detection:

```
src/
├── main.rs          # Application coordinator
├── deployment.rs    # Environment detection & deployment modes
├── commands.rs      # Command definitions and parsing
├── handlers.rs      # Message processing and Lambda handler
└── ai.rs           # Extensible AI backend system
```

### Deployment Modes

- **🔧 Polling Mode**: Local development with teloxide REPL
- **🌐 Webhook Mode**: Production with axum web server
- **☁️ Lambda Mode**: Serverless deployment on AWS Lambda

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ 
- Telegram Bot Token from [@BotFather](https://t.me/BotFather)
- OpenAI API Key (optional, for AI features)
- Alpha Vantage API Key (optional, for stock features)

### Local Development

1. **Clone the repository**
   ```bash
   git clone https://github.com/jzwdsb/telegram_bot.git
   cd telegram_bot
   ```

2. **Set up environment**
   ```bash
   cp .env.example .env
   # Edit .env and add your tokens:
   # - TELOXIDE_TOKEN (required) - Get from @BotFather on Telegram
   # - OPENAI_API_KEY (optional) - Get from https://platform.openai.com/api-keys
   # - ALPHA_VANTAGE_API_KEY (optional) - Get free key from https://www.alphavantage.co/support/#api-key
   ```

3. **Run locally**
   ```bash
   cargo run
   ```

The bot automatically detects it's running locally and uses polling mode.

## 📋 Available Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/help` | Show available commands | `/help` |
| `/username <name>` | Set username | `/username Alice` |
| `/usernameandage <name> <age>` | Set username and age | `/usernameandage Bob 25` |
| `/general <message>` | Chat with AI | `/general Hello, how are you?` |
| `/model <name>` | Change AI model | `/model list` or `/model gpt-4` |
| `/price <symbol>` | Get stock quote | `/price AAPL` |
| `/news <symbol>` | Get stock news | `/news TSLA` |

### Group Chat Usage

In group chats, mention the bot:
- `@yourbotname /help` - Get help
- `@yourbotname Hello!` - Direct AI chat (no `/general` needed)
- `@yourbotname` - Show available commands

## ☁️ AWS Lambda Deployment

### Prerequisites

- [AWS CLI](https://aws.amazon.com/cli/) configured
- [Terraform](https://terraform.io/downloads) installed
- [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda) installed
- [Zig](https://ziglang.org/) (for ARM64 cross-compilation)

```bash
# Install prerequisites
cargo install cargo-lambda
# Install Zig (macOS)
brew install zig
```

### Deploy to AWS

1. **Configure deployment**
   ```bash
   cd deployment/terraform
   cp terraform.tfvars.example terraform.tfvars
   # Edit terraform.tfvars with your tokens and AWS region
   ```

2. **Deploy with one command**
   ```bash
   ./deployment/deploy.sh
   ```

The deployment script automatically:
- ✅ Builds ARM64 Lambda binary
- ✅ Creates AWS infrastructure with Terraform
- ✅ Configures Telegram webhook
- ✅ Sets up CloudWatch logging

### Manual Deployment Steps

```bash
# Build Lambda function
cargo lambda build --release --target aarch64-unknown-linux-gnu --features lambda --no-default-features

# Deploy infrastructure
cd deployment/terraform
terraform init
terraform apply

# Set up webhook (automatic in deploy.sh)
WEBHOOK_URL=$(terraform output -raw webhook_url)
curl -X POST "https://api.telegram.org/bot$TELEGRAM_TOKEN/setWebhook" -d "url=$WEBHOOK_URL"
```

## 🔧 Configuration

### Environment Variables

| Variable | Description | Required | Example |
|----------|-------------|----------|---------|
| `TELOXIDE_TOKEN` | Telegram Bot Token | ✅ | `1234567890:ABC...` |
| `OPENAI_API_KEY` | OpenAI API Key (AI features) | ❌ | `sk-proj-...` |
| `ALPHA_VANTAGE_API_KEY` | Alpha Vantage API Key (stock features) | ❌ | `XXXXXXXXXXXXXXXX` |
| `WEBHOOK_URL` | Webhook URL (production) | ❌ | `https://example.com/webhook` |
| `RUST_LOG` | Log level | ❌ | `info` |

### Deployment Detection

The bot automatically detects the deployment environment:

- **Local**: Default polling mode
- **Production**: Webhook mode (when `PORT` + `WEBHOOK_URL` are set)
- **Lambda**: Serverless mode (when `AWS_LAMBDA_FUNCTION_NAME` exists)

## 🛠️ Development

### Project Structure

```
├── src/                 # Rust source code
├── deployment/         # Infrastructure as Code
│   ├── terraform/      # Terraform configuration
│   ├── deploy.sh      # Deployment script
│   └── destroy.sh     # Cleanup script
├── .env.example       # Environment template
└── Cargo.toml        # Rust dependencies
```

### Building

```bash
# Development build
cargo build

# Production build
cargo build --release

# Lambda build
cargo lambda build --release --target aarch64-unknown-linux-gnu --features lambda --no-default-features

# Check code quality
cargo clippy --all-features
```

### Testing

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --check
```

## 🤖 AI Integration

The bot features an extensible AI backend system:

```rust
#[async_trait]
trait AiBackend {
    async fn chat(&self, message: &str) -> Result<String, Box<dyn std::error::Error>>;
}
```

Currently supports:
- ✅ OpenAI ChatGPT (gpt-3.5-turbo)
- 🔄 Easy to extend for other providers

## 📊 Monitoring

### CloudWatch Logs (AWS)

```bash
# Follow logs in real-time
aws logs tail /aws/lambda/your-bot-name --follow

# View recent logs
aws logs tail /aws/lambda/your-bot-name --since 1h
```

### Local Logging

Set `RUST_LOG=debug` for detailed logging:

```bash
RUST_LOG=debug cargo run
```

## 🔒 Security

- ✅ Environment-based credential management
- ✅ No secrets in git history
- ✅ Proper `.gitignore` configuration
- ✅ Input validation and error handling
- ✅ Secure Terraform state management

## 🚧 Infrastructure

### AWS Resources Created

- **Lambda Function**: ARM64 runtime with custom handler
- **Function URL**: Public HTTPS endpoint for webhooks
- **IAM Role**: Minimal permissions for Lambda execution
- **CloudWatch Logs**: Centralized logging with retention policies

### Cost Optimization

- ARM64 architecture for better price/performance
- Minimal memory allocation (256MB)
- Short timeout (30 seconds)
- Efficient Rust binary (~6MB)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting (`cargo test && cargo clippy`)
5. Commit changes (`git commit -am 'Add amazing feature'`)
6. Push to branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [teloxide](https://github.com/teloxide/teloxide) - Rust Telegram bot framework
- [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
- [OpenAI API](https://openai.com/api/) - AI chat capabilities

## 📞 Support

- 🐛 [Report Issues](https://github.com/jzwdsb/telegram_bot/issues)
- 💬 [Discussions](https://github.com/jzwdsb/telegram_bot/discussions)
- 📖 [Documentation](./CLAUDE.md)

---

Made with ❤️ and 🦀 Rust