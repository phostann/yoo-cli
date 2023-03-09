use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{env, thread, time::Duration};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::loading::progress_bar;

mod create;
mod loading;
mod submit;

#[derive(Parser)]
#[command(name = "yoo")]
#[command(author = "phos")]
#[command(version = "0.1.0")]
#[command(
about = "An awesome CLI tool for frontend developers",
long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[arg(long)]
    server: Option<String>,

    #[arg(long)]
    gitlab_server: Option<String>,

    #[arg(long)]
    gitlab_token: Option<String>,

    #[arg(long)]
    pub_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// create a new project
    Create {},
    /// Upload the project
    Submit {},
}

/// init the cli
pub fn init() -> Result<()> {
    let mut cli = Cli::parse();

    let level = if cli.debug { Level::DEBUG } else { Level::INFO };

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber)
        .with_context(|| format!("Failed to set global default subscriber"))?;

    tracing::info!("Starting check the cli config");
    let pb = progress_bar("Checking")?;
    check_config(&mut cli)?;
    thread::sleep(Duration::from_secs(5));
    pb.finish_with_message("Check the cli config success");
    // print the configuration
    tracing::info!("Check the cli config success");
    tracing::info!("server_url: {}", cli.server.clone().unwrap());
    tracing::info!("gitlab_url: {}", cli.gitlab_server.clone().unwrap());
    tracing::info!("gitlab_token: {}", cli.gitlab_token.clone().unwrap());
    tracing::info!("pub_key: {}", cli.pub_key.clone().unwrap());

    match cli.command {
        Some(Commands::Create {}) => {
            return create::create(&cli);
        }
        Some(Commands::Submit {}) => {
            return submit::submit();
        }
        None => {
            println!("Nothing to do");
        }
    }

    Ok(())
}

/// check the server url , gitlab url and token of the gitlab
fn check_config(cli: &mut Cli) -> Result<()> {
    // if the cli doesn't confige the SERVER_URL, check the env
    if let None = cli.server.as_deref() {
        let server_url = env::var("YOO_SERVER").with_context(|| "SERVER is not set")?;
        cli.server = Some(server_url.clone());
    }

    // if the cli doesn't confige the GITLAB_SERVER, check the env
    if let None = cli.gitlab_server.as_deref() {
        let gitlab_url =
            env::var("YOO_GITLAB_SERVER").with_context(|| "GITLAB_SERVER is not set")?;
        cli.gitlab_server = Some(gitlab_url.clone());
    }

    // if the cli doesn't confige the GITLAB_TOKEN, check the env
    if let None = cli.gitlab_token.as_deref() {
        let gitlab_token =
            env::var("YOO_GITLAB_TOKEN").with_context(|| "GITLAB_TOKEN is not set")?;
        cli.gitlab_token = Some(gitlab_token.clone());
    }

    // if the cli doesn't config the PUBLIC_KEY, check the env
    if let None = cli.pub_key.as_deref() {
        let pub_key = env::var("YOO_PUBLIC_KEY").with_context(|| "PUBLIC_KEY is not set")?;
        cli.pub_key = Some(pub_key.clone());
    }

    Ok(())
}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
