use log::info;
use teloxide::prelude::*;

mod ai;
mod commands;
mod deployment;
mod handlers;
mod stock;
mod storage;

use deployment::{detect_deployment_mode, run_polling_mode, DeploymentMode};

#[cfg(feature = "lambda")]
use deployment::run_lambda_mode;

#[cfg(feature = "axum-server")]
use deployment::run_webhook_mode;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();
    info!("Starting telegram bot...");

    let bot = Bot::from_env();
    let deployment_mode = detect_deployment_mode();
    
    info!("🚀 Bot deployment detection: {deployment_mode}");

    let result = match deployment_mode {
        DeploymentMode::Lambda => {
            #[cfg(feature = "lambda")]
            {
                run_lambda_mode(bot).await
            }
            #[cfg(not(feature = "lambda"))]
            {
                panic!("Lambda environment detected but lambda feature not enabled. Compile with --features lambda");
            }
        }
        DeploymentMode::Webhook => {
            #[cfg(feature = "axum-server")]
            {
                run_webhook_mode(bot).await
            }
            #[cfg(not(feature = "axum-server"))]
            {
                panic!("Production environment detected but axum-server feature not enabled. Compile with --features axum-server");
            }
        }
        DeploymentMode::Polling => {
            run_polling_mode(bot).await;
            Ok(())
        }
    };

    if let Err(e) = result {
        panic!("Bot failed to start: {e}");
    }
}