use anyhow::{Context, Result};

use crate::cli::{self, CliCommand};
use crate::config::VeroConfig;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn run(raw_args: Vec<String>) -> Result<()> {
    let invocation = cli::parse(raw_args)?;

    match invocation.command {
        CliCommand::Help => {
            cli::print_help();
            Ok(())
        }
        CliCommand::Version => {
            println!("Vero v{} (Rust)", VERSION);
            Ok(())
        }
        CliCommand::Upgrade => upgrade(),
        CliCommand::Tui => {
            let config = VeroConfig::load().context("Failed to load config")?;
            crate::tui::run(config).await
        }
        _ => {
            let config = VeroConfig::load().context("Failed to load config")?;
            cli::execute(config, invocation).await
        }
    }
}

fn upgrade() -> Result<()> {
    println!("Upgrading vero...");
    let status = std::process::Command::new("cargo")
        .args(["install", "--git", "https://github.com/saravenpi/vero.git", "--force", "--quiet"])
        .status()
        .context("Failed to run cargo install")?;

    if status.success() {
        println!("vero upgraded successfully");
    } else {
        anyhow::bail!("upgrade failed");
    }
    Ok(())
}
