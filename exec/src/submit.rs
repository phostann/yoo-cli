use std::env;

use anyhow::{Context, Result};
use git::GitRepo;
use inquire::Select;

use crate::loading::loading;

pub(crate) fn submit() -> Result<()> {
    tracing::info!("Coming soon...");
    // prepare to submit
    // tracing::info!("Preparing to submit...");
    // let git_repo = check_repo()?;
    // push_repo(&git_repo)?;
    Ok(())
}

// check the current directory is a valid git project and has no uncommitted changes
fn check_repo() -> Result<GitRepo> {
    // get the current working dir
    let working_dir = env::current_dir().with_context(|| "Failed to get the current directory")?;
    let working_dir = working_dir
        .to_str()
        .with_context(|| "Failed to convert the current directory to a string")?;
    // 1. check if the current directory is a valid git project
    let git_repo = git::open_repo(working_dir)?;

    // 2. check if the current directory has uncommitted changes
    let has_uncommitted_changes = git_repo
        .has_uncommitted_changes()
        .with_context(|| "Failed to check if the current directory has uncommitted changes")?;

    if has_uncommitted_changes {
        return Err(anyhow::Error::msg(
            "The current directory has uncommitted changes",
        ));
    }
    Ok(git_repo)
}

// push the current directory to the remote repository
fn push_repo(git_repo: &GitRepo) -> Result<()> {
    // list the all feature branches and prompt user to select one
    let branches = git_repo.list_feature_branches()?;
    if branches.is_empty() {
        return Err(anyhow::Error::msg("No feature branch found"));
    }
    // let selected = Select::with_theme(&ColorfulTheme::default())
    //     .with_prompt("Please select a feature branch to submit")
    //     .items(&branches)
    //     .default(0)
    //     .interact()?;

    let selected: String = Select::new("Please select a feature branch to submit", branches)
        .prompt()
        .with_context(|| "Failed to interact with the user")?;

    // push the current directory to the remote repository
    tracing::info!("start push code");

    let pb = loading("Pushing")?;
    git_repo.push(selected.as_str())?;
    pb.finish();
    tracing::info!("Successfully push code to remote");

    Ok(())
}
