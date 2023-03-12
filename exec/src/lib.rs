use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

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
    gitlab_namespace_id: Option<u32>,

    #[arg(long)]
    pub_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project based on the template
    Create {},
    /// Submit the repo to the resource server
    Submit {},
}

/// init the cli
pub fn init() -> Result<()> {
    let mut cli = Cli::parse();

    let level = if cli.debug { Level::DEBUG } else { Level::INFO };

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber)
        .with_context(|| "Failed to set global default subscriber")?;

    if cli.debug {
        tracing::info!("CLI is running in debug mode");
    }
    // check the configuration
    check_config(&mut cli)?;
    // print the configuration
    tracing::info!("Successfully checked the configuration");
    tracing::info!("SERVER_URL: {}", cli.server.clone().unwrap());
    tracing::info!("GITLAB_URL: {}", cli.gitlab_server.clone().unwrap());
    tracing::info!("GITLAB_TOKEN: {}", cli.gitlab_token.clone().unwrap());
    tracing::info!(
        "GITLAB_NAMESPACE_ID: {}",
        cli.gitlab_namespace_id.unwrap()
    );
    tracing::info!("PUB_KEY: {}", cli.pub_key.clone().unwrap());

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
    // if the cli doesn't config the SERVER_URL, check the env
    if cli.server.is_none() {
        let server_url = env::var("YOO_SERVER").with_context(|| "SERVER is not set")?;
        cli.server = Some(server_url);
    }

    // if the cli doesn't config the GITLAB_SERVER, check the env
    if cli.gitlab_server.is_none() {
        let gitlab_url =
            env::var("YOO_GITLAB_SERVER").with_context(|| "GITLAB_SERVER is not set")?;
        cli.gitlab_server = Some(gitlab_url);
    }

    // if the cli doesn't config the GITLAB_TOKEN, check the env
    if cli.gitlab_token.is_none() {
        let gitlab_token =
            env::var("YOO_GITLAB_TOKEN").with_context(|| "GITLAB_TOKEN is not set")?;
        cli.gitlab_token = Some(gitlab_token);
    }

    // if the cli doesn't config the PUBLIC_KEY, check the env
    if cli.pub_key.is_none() {
        let pub_key = env::var("YOO_PUBLIC_KEY").with_context(|| "PUBLIC_KEY is not set")?;
        cli.pub_key = Some(pub_key);
    }

    // check namespace_id
    if cli.gitlab_namespace_id.is_none() {
        let namespace_id = env::var("YOO_GITLAB_NAMESPACE_ID")
            .with_context(|| "GITLAB_NAMESPACE_ID is not set")?
            .parse()
            .with_context(|| "GITLAB_NAMESPACE_ID is not a number")?;
        cli.gitlab_namespace_id = Some(namespace_id);
    }

    Ok(())
}
