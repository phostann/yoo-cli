use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use create::{login, read_authorization, write_authorization};
use once_cell::sync::Lazy;
use reqwest::StatusCode;
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

    /// The yoo server email
    #[arg(long)]
    server_email: Option<String>,

    /// The yoo server password
    #[arg(long)]
    server_password: Option<String>,

    project_id: Option<i32>,

    project_name: Option<String>,
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
    fn delete_project(&self) -> Result<()> {
        if self.project_id.is_none() {
            tracing::debug!("Project id is none");
            return Ok(());
        }

        let mut authorization = match read_authorization() {
            Ok(auth) => auth,
            Err(_) => {
                let auth = login(
                    self.server.as_ref().unwrap(),
                    self.server_email.as_ref().unwrap(),
                    self.server_password.as_ref().unwrap(),
                )?;
                write_authorization(&auth)?;
                auth
            }
        };

        let resp = delete_project_from_server(self, &authorization)?;

        match resp.status() {
            StatusCode::OK => Ok(()),
            StatusCode::BAD_REQUEST => {
                authorization = login(
                    self.server.as_ref().unwrap(),
                    self.server_email.as_ref().unwrap(),
                    self.server_password.as_ref().unwrap(),
                )?;

                write_authorization(&authorization)?;

                let resp = delete_project_from_server(self, &authorization)?;

                if resp.status() != StatusCode::OK {
                    return Err(anyhow::Error::msg("Failed to delete the project"));
                }

                Ok(())
            }
            _ => Err(anyhow::Error::msg("Failed to delete the project"))?,
        }

        // Ok(())
    }

    fn delete_local_repo(&self) -> Result<()> {
        if self.project_name.is_none() {
            return Ok(());
        }
        // get working dir
        let working_dir =
            env::current_dir().with_context(|| "Failed to get the current directory")?;
        // get the repo dir
        let repo_dir = working_dir.join(self.project_name.clone().unwrap());
        // remove the repo dir
        std::fs::remove_dir_all(repo_dir).with_context(|| "Failed to remove the repo dir")?;
        Ok(())
    }
}

fn delete_project_from_server(
    cli: &Cli,
    authorization: &String,
) -> Result<reqwest::blocking::Response, anyhow::Error> {
    let resp = REQUEST
        .delete(format!("/v1/projects/{}", cli.project_id.unwrap()))
        .header("Authorization", authorization)
        .send()
        .with_context(|| "Failed to delete the project")?;
    Ok(resp)
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
    tracing::info!("SERVER_USERNAME: {}", cli.server_email.clone().unwrap());
    // tracing::info!("SERVER_PASSWORD: {}", cli.server_password.clone().unwrap());

    match cli.command {
        Some(Commands::Create {}) => match create::create(&mut cli) {
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::error!("Failed to create the repo: {}", err);
                let pb = loading("Cleaning up...")?;
                // try to delete the remote repo
                cli.delete_project()?;
                // try to delete the local repo
                cli.delete_local_repo()?;

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

    // check username
    if cli.server_email.is_none() {
        let username = env::var("YOO_SERVER_EMAIL").with_context(|| "SERVER_EMAIL is not set")?;
        cli.server_email = Some(username);
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
