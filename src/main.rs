mod app;
mod cli;
mod config;
mod email;
mod email_file;
mod models;
mod services;
mod storage;
mod tui;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    app::run(std::env::args().skip(1).collect()).await
}
