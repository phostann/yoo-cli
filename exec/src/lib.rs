use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use once_cell::sync::Lazy;
use std::env;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, Layer};

use crate::loading::loading;

mod create;
mod loading;
mod submit;

pub const CACHE_DIR: &str = ".yoo";
pub const CACHE_FILE: &str = "cache.json";

// lazy to initialize the reqwest client
static REQUEST: Lazy<reqwest::blocking::Client> = Lazy::new(reqwest::blocking::Client::new);

#[derive(Parser)]
#[command(name = "yoo")]
#[command(author = "phos")]
#[command(version = "0.1.0")]
#[command(
about = "An awesome CLI tool for yotoo technology frontend developers",
long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable debug mode
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    /// The yoo server address
    #[arg(long)]
    server: Option<String>,

    /// The yoo server username
    #[arg(long)]
    server_username: Option<String>,

    /// The yoo server password
    #[arg(long)]
    server_password: Option<String>,

    /// The gitlab server address
    #[arg(long)]
    gitlab_server: Option<String>,

    /// The gitlab token
    #[arg(long)]
    gitlab_token: Option<String>,

    /// The gitlab group id
    #[arg(long)]
    gitlab_namespace_id: Option<u32>,

    repo_id: Option<i32>,

    repo_name: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project based on the template
    Create {},
    /// Submit the repo to the resource server
    Submit {
        /// Specify the branch to submit
        #[arg(long)]
        branch: Option<String>,
    },
}

impl Cli {
    fn delete_remote_repo(&self) -> Result<()> {
        if self.repo_id.is_none() {
            return Ok(());
        }
        REQUEST
            .delete(format!(
                "{}/api/v4/projects/{}",
                self.gitlab_server.clone().unwrap(),
                self.repo_id.unwrap()
            ))
            .header("PRIVATE-TOKEN", self.gitlab_token.clone().unwrap())
            .send()
            .with_context(|| "Failed to delete the repo")?;

        Ok(())
    }

    fn delete_local_repo(&self) -> Result<()> {
        if self.repo_name.is_none() {
            return Ok(());
        }
        // get working dir
        let working_dir =
            env::current_dir().with_context(|| "Failed to get the current directory")?;
        // get the repo dir
        let repo_dir = working_dir.join(self.repo_name.clone().unwrap());
        // remove the repo dir
        std::fs::remove_dir_all(repo_dir).with_context(|| "Failed to remove the repo dir")?;
        Ok(())
    }

    fn unregister(&self) -> Result<()> {
        if self.repo_name.is_none() {
            return Ok(());
        }
        Ok(())
    }
}

/// init the cli
pub fn init() -> Result<()> {
    loading("Initializing...")?.finish_and_clear();

    let mut cli = Cli::parse();

    let level_filter = if cli.debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    let enable_ansi = true;

    let layer = tracing_subscriber::fmt::Layer::new()
        .with_ansi(enable_ansi)
        .with_filter(level_filter);

    let subscriber = tracing_subscriber::registry().with(layer);

    tracing::subscriber::set_global_default(subscriber)
        .with_context(|| "Failed to set global default subscriber")?;

    if cli.debug {
        tracing::info!("CLI is running in debug mode");
    }
    // create cache dir and cache file
    create_cache_file()?;

    // check the configuration
    check_config(&mut cli)?;

    // print the configuration
    tracing::info!("Your configuration is as follows: ");
    tracing::info!("SERVER_URL: {}", cli.server.clone().unwrap());
    tracing::info!("GITLAB_URL: {}", cli.gitlab_server.clone().unwrap());
    tracing::info!("GITLAB_TOKEN: {}", cli.gitlab_token.clone().unwrap());
    tracing::info!("GITLAB_NAMESPACE_ID: {}", cli.gitlab_namespace_id.unwrap());
    tracing::info!("SERVER_USERNAME: {}", cli.server_username.clone().unwrap());
    // tracing::info!("SERVER_PASSWORD: {}", cli.server_password.clone().unwrap());

    match cli.command {
        Some(Commands::Create {}) => match create::create(&mut cli) {
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::error!("Failed to create the repo: {}", err);
                let pb = loading("Cleaning up...")?;
                // try to delete the remote repo
                cli.delete_remote_repo()?;
                // try to delete the local repo
                cli.delete_local_repo()?;
                // try to unregister the repo
                cli.unregister()?;
                pb.finish_and_clear();
                tracing::info!("Successfully cleaned up");
                if cli.debug {
                    Err(err)
                } else {
                    Ok(())
                }
            }
        },
        Some(Commands::Submit { ref branch }) => submit::submit(&cli, branch.clone()),
        None => Ok(()),
    }
}

// check the environment variable configuration
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

    // check namespace_id
    if cli.gitlab_namespace_id.is_none() {
        let namespace_id = env::var("YOO_GITLAB_NAMESPACE_ID")
            .with_context(|| "GITLAB_NAMESPACE_ID is not set")?
            .parse()
            .with_context(|| "GITLAB_NAMESPACE_ID is not a number")?;
        cli.gitlab_namespace_id = Some(namespace_id);
    }

    // check username
    if cli.server_username.is_none() {
        let username =
            env::var("YOO_SERVER_USERNAME").with_context(|| "SERVER_USERNAME is not set")?;
        cli.server_username = Some(username);
    }

    // check password
    if cli.server_password.is_none() {
        let password =
            env::var("YOO_SERVER_PASSWORD").with_context(|| "SERVER_PASSWORD is not set")?;
        cli.server_password = Some(password);
    }

    Ok(())
}

fn create_cache_file() -> Result<()> {
    let home_dir = dirs::home_dir().with_context(|| "Failed to get home dir")?;
    let cache_dir = home_dir.join(CACHE_DIR);
    if !cache_dir.exists() {
        // create the cache dir
        std::fs::create_dir(&cache_dir).with_context(|| "Failed to create cache dir")?;
    }

    let cache_file = cache_dir.join(CACHE_FILE);
    if !cache_file.exists() {
        // create the cache file
        std::fs::File::create(&cache_file).with_context(|| "Failed to create cache file")?;
    }

    Ok(())
}
