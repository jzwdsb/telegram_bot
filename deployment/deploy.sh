#!/bin/bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TERRAFORM_DIR="$SCRIPT_DIR/terraform"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if cargo-lambda is installed
    if ! command -v cargo-lambda &> /dev/null; then
        log_error "cargo-lambda is required but not installed."
        log_info "Install it with: cargo install cargo-lambda"
        exit 1
    fi
    
    # Check if terraform is installed
    if ! command -v terraform &> /dev/null; then
        log_error "Terraform is required but not installed."
        log_info "Install it from: https://terraform.io/downloads"
        exit 1
    fi
    
    # Check if AWS CLI is configured
    if ! aws sts get-caller-identity &> /dev/null; then
        log_error "AWS CLI is not configured or credentials are invalid."
        log_info "Configure it with: aws configure"
        exit 1
    fi
    
    # Check if terraform.tfvars exists
    if [[ ! -f "$TERRAFORM_DIR/terraform.tfvars" ]]; then
        log_error "terraform.tfvars not found."
        log_info "Copy terraform.tfvars.example to terraform.tfvars and configure it:"
        log_info "cp $TERRAFORM_DIR/terraform.tfvars.example $TERRAFORM_DIR/terraform.tfvars"
        exit 1
    fi
    
    log_success "All prerequisites met!"
}

build_lambda() {
    log_info "Building Lambda function..."
    cd "$PROJECT_ROOT"
    
    # Build for Lambda target (ARM64)
    cargo lambda build --release --target aarch64-unknown-linux-gnu --features lambda --no-default-features
    
    log_success "Lambda function built successfully!"
}

deploy_infrastructure() {
    log_info "Deploying infrastructure with Terraform..."
    cd "$TERRAFORM_DIR"
    
    # Initialize Terraform
    terraform init
    
    # Plan deployment
    log_info "Planning Terraform deployment..."
    terraform plan
    
    # Apply deployment
    log_info "Applying Terraform deployment..."
    terraform apply -auto-approve
    
    log_success "Infrastructure deployed successfully!"
}

setup_telegram_webhook() {
    log_info "Setting up Telegram webhook..."
    cd "$TERRAFORM_DIR"
    
    # Get webhook URL from Terraform output
    WEBHOOK_URL=$(terraform output -raw webhook_url)
    
    # Get Telegram token from terraform.tfvars
    TELEGRAM_TOKEN=$(grep 'telegram_token' terraform.tfvars | cut -d'"' -f2)
    
    if [[ -z "$TELEGRAM_TOKEN" || "$TELEGRAM_TOKEN" == "1234567890:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" ]]; then
        log_error "Please set a valid telegram_token in terraform.tfvars"
        exit 1
    fi
    
    # Set webhook
    RESPONSE=$(curl -s -X POST "https://api.telegram.org/bot$TELEGRAM_TOKEN/setWebhook" \
        -d "url=$WEBHOOK_URL" \
        -d "allowed_updates=[\"message\"]")
    
    if echo "$RESPONSE" | grep -q '"ok":true'; then
        log_success "Telegram webhook configured successfully!"
        log_info "Webhook URL: $WEBHOOK_URL"
    else
        log_error "Failed to set Telegram webhook:"
        echo "$RESPONSE"
        exit 1
    fi
}

show_deployment_info() {
    log_info "Deployment Information:"
    cd "$TERRAFORM_DIR"
    
    echo
    terraform output deployment_info
    echo
    
    log_info "Bot is now deployed and ready to use!"
    log_info "Check logs with: aws logs tail $(terraform output -raw cloudwatch_log_group) --follow"
}

main() {
    log_info "🚀 Starting Telegram Bot deployment..."
    
    check_prerequisites
    build_lambda
    deploy_infrastructure
    setup_telegram_webhook
    show_deployment_info
    
    log_success "🎉 Deployment completed successfully!"
}

# Show help
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Deploy Telegram bot to AWS Lambda"
    echo
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -b, --build    Only build the Lambda function"
    echo "  -d, --deploy   Only deploy infrastructure (skip build)"
    echo "  -w, --webhook  Only setup webhook (skip build and deploy)"
    echo
    echo "Prerequisites:"
    echo "  - cargo-lambda (cargo install cargo-lambda)"
    echo "  - terraform (https://terraform.io/downloads)"
    echo "  - AWS CLI configured (aws configure)"
    echo "  - terraform.tfvars configured"
}

# Parse command line arguments
case "${1:-}" in
    -h|--help)
        show_help
        exit 0
        ;;
    -b|--build)
        check_prerequisites
        build_lambda
        ;;
    -d|--deploy)
        check_prerequisites
        deploy_infrastructure
        ;;
    -w|--webhook)
        check_prerequisites
        setup_telegram_webhook
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        show_help
        exit 1
        ;;
esac