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

confirm_destruction() {
    echo
    log_warning "⚠️  This will destroy ALL infrastructure resources for the Telegram bot!"
    log_warning "This action cannot be undone."
    echo
    
    read -p "Are you sure you want to continue? Type 'yes' to confirm: " confirmation
    
    if [[ "$confirmation" != "yes" ]]; then
        log_info "Destruction cancelled."
        exit 0
    fi
}

remove_telegram_webhook() {
    log_info "Removing Telegram webhook..."
    cd "$TERRAFORM_DIR"
    
    # Check if terraform state exists
    if [[ ! -f "terraform.tfstate" ]]; then
        log_warning "No Terraform state found. Skipping webhook removal."
        return
    fi
    
    # Get Telegram token from terraform.tfvars
    if [[ -f "terraform.tfvars" ]]; then
        TELEGRAM_TOKEN=$(grep 'telegram_token' terraform.tfvars | cut -d'"' -f2 2>/dev/null || echo "")
        
        if [[ -n "$TELEGRAM_TOKEN" && "$TELEGRAM_TOKEN" != "1234567890:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" ]]; then
            # Remove webhook
            RESPONSE=$(curl -s -X POST "https://api.telegram.org/bot$TELEGRAM_TOKEN/deleteWebhook")
            
            if echo "$RESPONSE" | grep -q '"ok":true'; then
                log_success "Telegram webhook removed successfully!"
            else
                log_warning "Failed to remove Telegram webhook (this is usually not critical):"
                echo "$RESPONSE"
            fi
        else
            log_warning "Could not find valid Telegram token. Skipping webhook removal."
        fi
    else
        log_warning "terraform.tfvars not found. Skipping webhook removal."
    fi
}

destroy_infrastructure() {
    log_info "Destroying infrastructure with Terraform..."
    cd "$TERRAFORM_DIR"
    
    # Check if terraform is initialized
    if [[ ! -d ".terraform" ]]; then
        log_info "Initializing Terraform..."
        terraform init
    fi
    
    # Show what will be destroyed
    log_info "Planning destruction..."
    terraform plan -destroy
    
    echo
    log_warning "The above resources will be DESTROYED!"
    read -p "Continue with destruction? Type 'yes' to confirm: " final_confirmation
    
    if [[ "$final_confirmation" != "yes" ]]; then
        log_info "Destruction cancelled."
        exit 0
    fi
    
    # Destroy infrastructure
    log_info "Destroying infrastructure..."
    terraform destroy -auto-approve
    
    log_success "Infrastructure destroyed successfully!"
}

cleanup_files() {
    log_info "Cleaning up deployment files..."
    cd "$TERRAFORM_DIR"
    
    # Remove terraform state files (optional)
    read -p "Remove Terraform state files? This will make it impossible to manage existing resources. (y/N): " remove_state
    
    if [[ "$remove_state" =~ ^[Yy]$ ]]; then
        rm -f terraform.tfstate*
        rm -f .terraform.lock.hcl
        rm -rf .terraform/
        log_success "Terraform state files removed."
    fi
    
    # Remove built lambda zip
    if [[ -f "telegram_bot.zip" ]]; then
        rm -f telegram_bot.zip
        log_info "Removed Lambda deployment package."
    fi
}

main() {
    log_info "🗑️  Starting Telegram Bot infrastructure destruction..."
    
    confirm_destruction
    remove_telegram_webhook
    destroy_infrastructure
    cleanup_files
    
    log_success "🎉 Destruction completed successfully!"
    log_info "All AWS resources have been removed."
}

# Show help
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Destroy Telegram bot AWS infrastructure"
    echo
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -f, --force    Skip confirmation prompts (use with caution!)"
    echo
    echo "This script will:"
    echo "  1. Remove the Telegram webhook"
    echo "  2. Destroy all AWS resources (Lambda, IAM roles, CloudWatch logs)"
    echo "  3. Optionally clean up Terraform state files"
}

# Parse command line arguments
case "${1:-}" in
    -h|--help)
        show_help
        exit 0
        ;;
    -f|--force)
        log_warning "Force mode enabled - skipping confirmations"
        remove_telegram_webhook
        destroy_infrastructure
        cleanup_files
        log_success "🎉 Destruction completed successfully!"
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