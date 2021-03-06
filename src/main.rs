#![allow(dead_code)]
mod auth;
mod network;
mod protocol;
mod service;
mod util;

use log::error;
use sqlx::SqlitePool;
use std::time::Duration;

pub type EnvData = sqlx::SqlitePool;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    /* Initialize a logger */
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    /* Connect (Open) database */
    let pool = SqlitePool::new("sqlite:lock.db").await.unwrap();
    let pool2 = pool.clone();

    async_std::task::spawn(async move {
        if let Err(e) = service::serve(pool).await {
            error!("Failed to run HTTP server: {}", e);
        }
    });

    async_std::task::spawn(async move {
        if let Err(e) = network::run("0.0.0.0:10292", pool2).await {
            error!("Lock server exited because {}", e);
        }
    });

    loop {
        // Do nothing
        async_std::task::sleep(Duration::from_secs(1)).await;
    }
}
