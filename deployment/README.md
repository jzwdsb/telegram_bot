# Telegram Bot AWS Deployment

This directory contains Terraform infrastructure and deployment scripts for deploying the Telegram bot to AWS Lambda with public webhook access.

## 🏗️ Architecture

The deployment creates the following AWS resources:

- **AWS Lambda Function**: Runs the Rust bot with custom runtime
- **Lambda Function URL**: Provides public HTTPS endpoint for webhooks  
- **IAM Role & Policies**: Minimal permissions for Lambda execution
- **CloudWatch Log Group**: Centralized logging with configurable retention

## 📋 Prerequisites

Before deploying, ensure you have:

1. **cargo-lambda** - For building Rust Lambda functions
   ```bash
   cargo install cargo-lambda
   ```

2. **Terraform** - Infrastructure as Code tool
   ```bash
   # macOS
   brew install terraform
   
   # Or download from https://terraform.io/downloads
   ```

3. **AWS CLI** - Configured with appropriate credentials
   ```bash
   aws configure
   # Enter your Access Key ID, Secret Access Key, and region
   ```

4. **Telegram Bot Token** - Create a bot with @BotFather on Telegram

## 🚀 Quick Deployment

### 1. Configure Variables

Copy the example configuration and edit it:

```bash
cp deployment/terraform/terraform.tfvars.example deployment/terraform/terraform.tfvars
```

Edit `deployment/terraform/terraform.tfvars`:

```hcl
# AWS Configuration
aws_region  = "us-west-2"           # Your preferred AWS region
environment = "prod"                # Environment name

# Bot Configuration  
bot_name        = "my-telegram-bot" # Lambda function name
telegram_token  = "123456:ABC..."   # From @BotFather
openai_api_key  = "sk-..."          # Optional: for AI features

# Logging
log_level           = "info"        # error, warn, info, debug, trace
log_retention_days  = 14           # CloudWatch log retention
```

### 2. Deploy Everything

Run the automated deployment script:

```bash
./deployment/deploy.sh
```

This script will:
1. ✅ Check all prerequisites  
2. 🔨 Build the Lambda function with `cargo lambda`
3. 🏗️ Deploy infrastructure with Terraform
4. 🔗 Configure Telegram webhook automatically
5. 📊 Show deployment information

### 3. Verify Deployment

After deployment, test your bot:
1. Message your bot on Telegram
2. Check logs: `aws logs tail /aws/lambda/your-bot-name --follow`

## 🛠️ Manual Operations

### Individual Operations

**Build only:**
```bash
./deployment/deploy.sh --build
```

**Deploy infrastructure only:**
```bash
./deployment/deploy.sh --deploy
```

**Setup webhook only:**
```bash
./deployment/deploy.sh --webhook
```

### Manual Terraform Commands

```bash
cd deployment/terraform

# Initialize Terraform
terraform init

# Plan deployment
terraform plan

# Apply changes
terraform apply

# Show outputs
terraform output

# Destroy everything
terraform destroy
```

## 🗑️ Destroying Infrastructure

To remove all AWS resources:

```bash
./deployment/destroy.sh
```

Or force destroy without prompts:
```bash
./deployment/destroy.sh --force
```

## 📊 Monitoring & Logs

### View Logs
```bash
# Real-time logs
aws logs tail /aws/lambda/your-bot-name --follow

# Recent logs
aws logs tail /aws/lambda/your-bot-name --since 1h
```

### CloudWatch Dashboard
Access logs via AWS Console:
1. Go to CloudWatch → Log groups
2. Find `/aws/lambda/your-bot-name`
3. View log streams and metrics

## 🔧 Configuration Options

### Terraform Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `aws_region` | AWS deployment region | `us-west-2` | No |
| `environment` | Environment name | `prod` | No |
| `bot_name` | Lambda function name | `telegram-bot` | No |
| `telegram_token` | Telegram bot token | - | **Yes** |
| `openai_api_key` | OpenAI API key | `""` | No |
| `log_level` | Rust log level | `info` | No |
| `log_retention_days` | Log retention period | `14` | No |

### Environment Variables (Set automatically)

The Lambda function receives these environment variables:

- `RUST_LOG` - Logging level 
- `TELOXIDE_TOKEN` - Telegram bot token
- `OPENAI_API_KEY` - OpenAI API key (if provided)
- `WEBHOOK_URL` - Auto-generated Lambda function URL
- `AWS_LAMBDA_FUNCTION_NAME` - Lambda function name (AWS managed)

## 🔒 Security Considerations

### IAM Permissions
The Lambda function has minimal IAM permissions:
- Basic Lambda execution role
- CloudWatch logs write access
- No additional AWS service permissions

### Network Security
- Lambda Function URL is publicly accessible (required for webhooks)
- CORS configured for POST requests only
- No VPC configuration (uses default Lambda network isolation)

### Secrets Management
- Telegram token and OpenAI API key are marked as sensitive
- Consider using AWS Secrets Manager for production deployments

## 🐛 Troubleshooting

### Common Issues

**1. Build fails with cargo-lambda error:**
```bash
# Install cargo-lambda
cargo install cargo-lambda

# Or update if already installed
cargo install --force cargo-lambda
```

**2. AWS credentials not configured:**
```bash
aws configure
# Or set environment variables:
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret
export AWS_DEFAULT_REGION=us-west-2
```

**3. Telegram webhook setup fails:**
- Verify telegram_token in terraform.tfvars
- Check if Lambda function URL is accessible
- Ensure bot token has correct permissions

**4. Lambda function not responding:**
- Check CloudWatch logs for errors
- Verify environment variables are set correctly
- Test Lambda function directly in AWS Console

### Debug Commands

```bash
# Check AWS credentials
aws sts get-caller-identity

# Verify Terraform state
cd deployment/terraform && terraform show

# Test webhook URL
curl -X POST "$(terraform output -raw webhook_url)" \
  -H "Content-Type: application/json" \
  -d '{"test": "message"}'

# Manual webhook setup
curl -X POST "https://api.telegram.org/bot$TELEGRAM_TOKEN/setWebhook" \
  -d "url=$(terraform output -raw webhook_url)"
```

## 💰 Cost Estimation

Estimated monthly costs for moderate usage:

- **Lambda Function**: ~$0.20-$2.00 (depending on usage)
- **CloudWatch Logs**: ~$0.50-$5.00 (depending on log volume)
- **Data Transfer**: ~$0.10-$1.00 (webhook traffic)

**Total: ~$1-$8/month** for typical bot usage

Free tier eligible for new AWS accounts.

## 🔄 Updates & Maintenance

### Updating Bot Code
1. Make changes to Rust code
2. Run `./deployment/deploy.sh` to rebuild and redeploy

### Updating Infrastructure
1. Modify Terraform files in `deployment/terraform/`
2. Run `terraform plan` to preview changes
3. Run `terraform apply` to apply changes

### Backup Recommendations
- Export Terraform state: `terraform show > backup.txt`
- Save terraform.tfvars in secure location
- Document any manual AWS console changes

## 📚 Additional Resources

- [Telegram Bot API Documentation](https://core.telegram.org/bots/api)
- [AWS Lambda Developer Guide](https://docs.aws.amazon.com/lambda/)
- [Terraform AWS Provider Documentation](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)
- [cargo-lambda Documentation](https://cargo-lambda.info/)