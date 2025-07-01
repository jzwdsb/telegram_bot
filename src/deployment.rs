use std::env;
use log::info;
use teloxide::prelude::*;

#[cfg(feature = "axum-server")]
use axum::{routing::get, routing::post, Router};

#[cfg(feature = "lambda")]
use lambda_runtime::service_fn;

use crate::handlers::handle_message;

#[cfg(feature = "lambda")]
use crate::handlers::lambda_handler;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentMode {
    Lambda,
    Webhook,
    Polling,
}

impl std::fmt::Display for DeploymentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentMode::Lambda => write!(f, "AWS LAMBDA"),
            DeploymentMode::Webhook => write!(f, "WEBHOOK (Production)"),
            DeploymentMode::Polling => write!(f, "POLLING (Development)"),
        }
    }
}

pub fn is_lambda_environment() -> bool {
    // Check if running on AWS Lambda
    env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() ||
    env::var("LAMBDA_RUNTIME_API").is_ok() ||
    // Manual override
    env::var("LAMBDA_MODE").map(|v| v == "true").unwrap_or(false)
}

pub fn is_production_environment() -> bool {
    // Check common production environment indicators
    env::var("RAILWAY_ENVIRONMENT").is_ok() ||
    env::var("HEROKU_APP_NAME").is_ok() ||
    env::var("VERCEL").is_ok() ||
    env::var("NODE_ENV").map(|v| v == "production").unwrap_or(false) ||
    env::var("ENVIRONMENT").map(|v| v == "production").unwrap_or(false) ||
    env::var("DEPLOYMENT_ENV").map(|v| v == "production").unwrap_or(false) ||
    // Check if PORT is set by cloud provider (common pattern)
    (env::var("PORT").is_ok() && env::var("WEBHOOK_URL").is_ok()) ||
    // Manual override
    env::var("WEBHOOK_MODE").map(|v| v == "true").unwrap_or(false) ||
    // Lambda is also production
    is_lambda_environment()
}

pub fn detect_deployment_mode() -> DeploymentMode {
    if is_lambda_environment() {
        DeploymentMode::Lambda
    } else if is_production_environment() {
        DeploymentMode::Webhook
    } else {
        DeploymentMode::Polling
    }
}

#[cfg(feature = "lambda")]
pub async fn run_lambda_mode(bot: Bot) -> Result<(), Box<dyn std::error::Error>> {
    info!("☁️ AWS Lambda environment detected - setting up Lambda runtime");
    
    // Set up webhook URL if provided
    if let Ok(webhook_url) = env::var("WEBHOOK_URL") {
        info!("🔗 Setting up webhook at: {webhook_url}");
        bot.set_webhook(webhook_url.parse()?)
            .await
            .map_err(|e| format!("Failed to set webhook: {e}"))?;
    }
    
    info!("👂 Lambda handler ready to receive updates!");
    lambda_runtime::run(service_fn(lambda_handler))
        .await
        .map_err(|e| format!("Lambda runtime failed: {e}").into())
}

#[cfg(feature = "axum-server")]
pub async fn run_webhook_mode(bot: Bot) -> Result<(), Box<dyn std::error::Error>> {
    use axum::extract::State;
    use axum::response::Html;
    use axum::Json;
    
    async fn health_check() -> Html<&'static str> {
        Html("<h1>Bot is running!</h1>")
    }

    async fn webhook_handler(
        State(bot): State<Bot>,
        Json(update): Json<teloxide::types::Update>,
    ) -> &'static str {
        info!("🔗 Webhook received update: {:?}", update.id);

        if let teloxide::types::UpdateKind::Message(message) = update.kind {
            let _ = handle_message(bot, message).await;
        } else {
            info!("🔄 Received non-message update in webhook");
        }
        "OK"
    }

    let webhook_url = env::var("WEBHOOK_URL")
        .map_err(|_| "WEBHOOK_URL must be set for webhook mode")?;
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .map_err(|_| "PORT must be a valid number")?;

    info!("🌐 Production environment detected - running in WEBHOOK mode");
    info!("🔗 Setting up webhook at: {webhook_url}");

    bot.set_webhook(webhook_url.parse()?)
        .await
        .map_err(|e| format!("Failed to set webhook: {e}"))?;

    let app = Router::new()
        .route("/", get(health_check))
        .route("/webhook", post(webhook_handler))
        .with_state(bot);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .map_err(|e| format!("Failed to bind to port: {e}"))?;

    info!("👂 Webhook server listening on port {port} - ready to receive updates!");
    
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server failed: {e}").into())
}

pub async fn run_polling_mode(bot: Bot) {
    info!("🔄 Development environment detected - running in POLLING mode");
    info!("👂 Starting polling loop - ready to receive updates!");

    // Use message handler that properly handles group chats
    let handler = Update::filter_message().endpoint(handle_message);
    Dispatcher::builder(bot, handler).build().dispatch().await;
}