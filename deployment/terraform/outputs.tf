output "lambda_function_name" {
  description = "Name of the Lambda function"
  value       = aws_lambda_function.telegram_bot.function_name
}

output "lambda_function_arn" {
  description = "ARN of the Lambda function"
  value       = aws_lambda_function.telegram_bot.arn
}

output "webhook_url" {
  description = "Public webhook URL for Telegram bot"
  value       = aws_lambda_function_url.telegram_bot_url.function_url
}

output "cloudwatch_log_group" {
  description = "CloudWatch log group name"
  value       = aws_cloudwatch_log_group.lambda_logs.name
}

output "telegram_webhook_setup_command" {
  description = "Command to set up Telegram webhook"
  value       = "curl -X POST https://api.telegram.org/bot${var.telegram_token}/setWebhook -d 'url=${aws_lambda_function_url.telegram_bot_url.function_url}'"
  sensitive   = true
}

output "deployment_info" {
  description = "Deployment summary"
  value = {
    region               = var.aws_region
    environment         = var.environment
    lambda_function     = aws_lambda_function.telegram_bot.function_name
    webhook_url         = aws_lambda_function_url.telegram_bot_url.function_url
    log_group          = aws_cloudwatch_log_group.lambda_logs.name
    bot_name           = var.bot_name
  }
}