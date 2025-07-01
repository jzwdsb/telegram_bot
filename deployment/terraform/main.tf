terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    archive = {
      source  = "hashicorp/archive"
      version = "~> 2.0"
    }
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "telegram-bot"
      Environment = var.environment
      ManagedBy   = "terraform"
    }
  }
}

# Archive the Lambda function code
data "archive_file" "lambda_zip" {
  type        = "zip"
  source_dir  = "${path.module}/../../target/lambda/telegram_bot"
  output_path = "${path.module}/telegram_bot.zip"
  depends_on  = [null_resource.build_lambda]
}

# Build the Lambda function
resource "null_resource" "build_lambda" {
  triggers = {
    # Rebuild when source code changes
    source_hash = filebase64sha256("${path.module}/../../src/main.rs")
  }

  provisioner "local-exec" {
    command = "cd ${path.module}/../.. && cargo lambda build --release"
  }
}

# IAM role for Lambda function
resource "aws_iam_role" "lambda_role" {
  name = "${var.bot_name}-lambda-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
}

# IAM policy for Lambda basic execution
resource "aws_iam_role_policy_attachment" "lambda_basic_execution" {
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
  role       = aws_iam_role.lambda_role.name
}

# Lambda function
resource "aws_lambda_function" "telegram_bot" {
  filename      = data.archive_file.lambda_zip.output_path
  function_name = var.bot_name
  role          = aws_iam_role.lambda_role.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]
  timeout       = 30
  memory_size   = 256

  source_code_hash = data.archive_file.lambda_zip.output_base64sha256

  environment {
    variables = {
      RUST_LOG       = var.log_level
      TELOXIDE_TOKEN = var.telegram_token
      OPENAI_API_KEY = var.openai_api_key
      # WEBHOOK_URL will be set after deployment via Lambda update
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.lambda_basic_execution,
    aws_cloudwatch_log_group.lambda_logs
  ]
}

# Lambda function URL for public webhook access
resource "aws_lambda_function_url" "telegram_bot_url" {
  function_name      = aws_lambda_function.telegram_bot.function_name
  authorization_type = "NONE"

  cors {
    allow_credentials = false
    allow_origins     = ["*"]
    allow_methods     = ["POST"]
    allow_headers     = ["date", "keep-alive"]
    expose_headers    = ["date", "keep-alive"]
    max_age           = 86400
  }
}

# CloudWatch Log Group
resource "aws_cloudwatch_log_group" "lambda_logs" {
  name              = "/aws/lambda/${var.bot_name}"
  retention_in_days = var.log_retention_days
}

# Lambda permission for function URL
resource "aws_lambda_permission" "allow_function_url" {
  statement_id  = "AllowFunctionUrlInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.telegram_bot.function_name
  principal     = "*"
  source_arn    = "${aws_lambda_function.telegram_bot.arn}/*"
}

# Update Lambda function with webhook URL after function URL is created
resource "null_resource" "update_webhook_url" {
  depends_on = [
    aws_lambda_function.telegram_bot,
    aws_lambda_function_url.telegram_bot_url
  ]

  triggers = {
    function_url = aws_lambda_function_url.telegram_bot_url.function_url
  }

  provisioner "local-exec" {
    command = <<-EOF
      aws lambda update-function-configuration \
        --function-name ${aws_lambda_function.telegram_bot.function_name} \
        --environment Variables="{RUST_LOG=${var.log_level},TELOXIDE_TOKEN=${var.telegram_token},OPENAI_API_KEY=${var.openai_api_key},WEBHOOK_URL=${aws_lambda_function_url.telegram_bot_url.function_url}}" \
        --region ${var.aws_region}
    EOF
  }
}
