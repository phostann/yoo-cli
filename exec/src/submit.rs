use anyhow::{Context, Ok, Result};
use dialoguer::{theme::ColorfulTheme, Select};

pub(crate) fn submit() -> Result<()> {
    // prepare to submit
    check_repo()?;
    push_repo()?;
    Ok(())
}

/// check the current directory is a valid git project and has no uncommitted changes
fn check_repo() -> Result<()> {
    // 1. check if the current directory is a valid git project
    let is_git_repo = git::is_git_project()
        .with_context(|| "Failed to check if the current directory is a valid git project")?;

    if !is_git_repo {
        return Err(anyhow::Error::msg(
            "The current directory is not a valid git project",
        ));
    }

    // 2. check if the current directory has uncommitted changes
    let has_uncommitted_changes = git::has_uncommitted_changes()
        .with_context(|| "Failed to check if the current directory has uncommitted changes")?;

    if has_uncommitted_changes {
        return Err(anyhow::Error::msg(
            "The current directory has uncommitted changes",
        ));
    }

    // list the all feature branches and propmt user to select one
    let branches = git::list_feature_branches()?;
    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please select a feature branch to submit")
        .items(&branches)
        .default(0)
        .interact()?;

    // push the current directory to the remote repository
    tracing::info!("start push code");
    git::push(&branches[selected])?;
    tracing::info!("Successfully push code to remote");

    Ok(())
}

/// push the current directory to the remote repository
fn push_repo() -> Result<()> {
    Ok(())
}
