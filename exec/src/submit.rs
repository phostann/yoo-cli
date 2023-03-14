use anyhow::Result;
use inquire::Select;

use crate::{loading, Cli};

pub(crate) fn submit(cli: &Cli, branch: Option<String>) -> Result<()> {
    // check if the current dir is a git repo
    let repo = git::open_repo(
        ".",
        cli.gitlab_username.clone().unwrap(),
        cli.gitlab_password.clone().unwrap(),
    )?;

    // check if there is uncommitted changes
    if repo.has_uncommitted_changes()? {
        tracing::error!("There are uncommitted changes, please commit them first");
        return Ok(());
    }

    let branch = match branch {
        Some(branch) => branch,
        None => {
            let branches = repo.list_branches()?;
            Select::new("Select a branch to submit", branches).prompt()?
        }
    };

    tracing::info!("Submitting the branch: {}", branch);

    // push the branch to the remote
    let pb = loading("Pushing")?;
    repo.push(&branch)?;
    pb.finish_and_clear();

    tracing::info!("Successfully pushed the branch to the remote");

    Ok(())
}
