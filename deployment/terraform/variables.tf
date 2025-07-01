variable "aws_region" {
  description = "AWS region for deployment"
  type        = string
  default     = "us-west-2"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "prod"
}

variable "bot_name" {
  description = "Name of the Telegram bot (used for Lambda function name)"
  type        = string
  default     = "telegram-bot"
  
  validation {
    condition     = can(regex("^[a-zA-Z0-9-_]+$", var.bot_name))
    error_message = "Bot name must contain only alphanumeric characters, hyphens, and underscores."
  }
}

variable "telegram_token" {
  description = "Telegram Bot API token from @BotFather"
  type        = string
  sensitive   = true
  
  validation {
    condition     = can(regex("^[0-9]+:[a-zA-Z0-9_-]+$", var.telegram_token))
    error_message = "Telegram token must be in format 'bot_id:token'."
  }
}

variable "openai_api_key" {
  description = "OpenAI API key for AI chat functionality"
  type        = string
  sensitive   = true
  default     = ""
  
  validation {
    condition     = var.openai_api_key == "" || can(regex("^sk-proj-[a-zA-Z0-9_-]+$", var.openai_api_key)) || can(regex("^sk-[a-zA-Z0-9_-]+$", var.openai_api_key))
    error_message = "OpenAI API key must start with 'sk-' or 'sk-proj-' followed by alphanumeric characters, underscores, and hyphens."
  }
}

variable "log_level" {
  description = "Rust log level (error, warn, info, debug, trace)"
  type        = string
  default     = "info"
  
  validation {
    condition     = contains(["error", "warn", "info", "debug", "trace"], var.log_level)
    error_message = "Log level must be one of: error, warn, info, debug, trace."
  }
}

variable "log_retention_days" {
  description = "CloudWatch log retention period in days"
  type        = number
  default     = 14
  
  validation {
    condition = contains([
      1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, 3653
    ], var.log_retention_days)
    error_message = "Log retention days must be a valid CloudWatch retention period."
  }
}