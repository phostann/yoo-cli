use anyhow::{Context, Result};
use console::{style, Emoji};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use serde::Deserialize;
use std::{
    env,
    fs::{remove_dir_all, remove_file, DirEntry},
    io::Error,
    path::PathBuf,
};

use crate::Cli;

#[derive(Debug, Deserialize)]
struct Response {
    code: i32,
    // msg: String,
    data: ResponseData,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    content: Vec<Template>,
}

#[derive(Debug, Deserialize)]
struct Template {
    name: String,
    repo: String,
    brief: String,
}

// check the current directory and ask the user if they want to continue
pub(crate) fn create(cli: &Cli) -> Result<()> {
    // get working directory
    let working_dir = env::current_dir().with_context(|| "Failed to get working director")?;

    let dir = working_dir
        .read_dir()
        .with_context(|| "Failed to read the current directory")?;

    let arr = dir.collect::<Vec<Result<DirEntry, Error>>>();
    // check if the current directory is empty
    if !arr.is_empty() {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "{}  The current directory is not empty. Do you want to cleanup and continue?",
                Emoji("⚠️", style("!!!").red().to_string().as_ref())
            ))
            .default(false)
            .wait_for_newline(true)
            .interact()
            .with_context(|| "Failed to interact with the user")?
        {
            for ele in arr {
                let path = ele.with_context(|| "Failed to unwrap the element")?;
                let path = path.path();
                if path.is_dir() {
                    remove_dir_all(path).with_context(|| "Failed to remove the directory")?;
                } else {
                    remove_file(path).with_context(|| "Failed to remove the file")?;
                }
            }
        } else {
            // return error
            return Err(anyhow::Error::msg("User canceled the operation"));
        }
    }

    let server_url = cli
        .server
        .as_deref()
        .with_context(|| "SERVER is not set")?;

    // show the project list
    let resp = reqwest::blocking::get(server_url).with_context(|| "Failed to get the templates")?;

    if resp.status() != 200 {
        return Err(anyhow::Error::msg("Failed to get the templates"));
    }

    let resp = resp
        .json::<Response>()
        .with_context(|| "Failed to parse the response")?;
    if resp.code != 0 {
        return Err(anyhow::Error::msg("Failed to get the templates"));
    }
    if resp.data.content.is_empty() {
        return Err(anyhow::Error::msg("No templates found"));
    }
    // show the options of the templates

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(
            resp.data
                .content
                .iter()
                .map(|x| format!("{} -- {}", x.name, x.brief))
                .collect::<Vec<String>>()
                .as_slice(),
        )
        .default(0)
        .interact()
        .with_context(|| "Failed to interact with the user")?;

    let seclection = &resp.data.content[selection];

    // clone the repo
    tracing::info!("Cloning the the {} into current directory", seclection.repo);
    git::clone(&seclection.repo)?;

    // remove the .git directory
    let git_dir = PathBuf::from(".git");
    if git_dir.exists() && git_dir.is_dir() {
        remove_dir_all(git_dir).with_context(|| "Failed to remove the .git directory")?;
    }

    // init the git repo
    git::init()?;
    tracing::info!("Git repository initialized");

    Ok(())
    // use gitlab api to create a new repo and cache the info
}
