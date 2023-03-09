use std::env;

use anyhow::{Context, Result};
use git2::{Cred, Repository};

pub fn clone(repo: &str) -> Result<()> {
    Repository::clone(repo, ".").with_context(|| "Failed to clone git repository")?;
    Ok(())
}

// git init command
pub fn init() -> Result<()> {
    Repository::init(".").with_context(|| "Failed to init git repository")?;
    Ok(())
}

pub fn is_git_project() -> Result<bool> {
    Repository::open(".").with_context(|| "Failed to open the repository")?;
    Ok(true)
}

pub fn has_uncommitted_changes() -> Result<bool> {
    let repo = Repository::open(".").with_context(|| "Failed to open the repository")?;

    let statuses = repo
        .statuses(None)
        .with_context(|| "Failed to get the status of the repository")?;

    if !statuses.is_empty() {
        return Ok(true);
    }

    Ok(false)
}

pub fn push(branch: &str) -> Result<()> {
    let repo = Repository::open(".").with_context(|| "Failed to open the repository")?;
    let mut remote = repo
        .find_remote("origin")
        .with_context(|| "Failed to push the repository")?;

    let branch = repo
        .find_branch(branch, git2::BranchType::Local)
        .with_context(|| "Failed to find the branch")?;

    let refs = branch.into_reference();

    let name = refs.name().with_context(|| "The reference name is none")?;

    let mut callbacks = git2::RemoteCallbacks::new();

    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        tracing::debug!("username_from_url: {:?}", username_from_url);
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
            None,
        )
    });

    let mut options = git2::PushOptions::new();

    options.remote_callbacks(callbacks);

    // push the code to master branch
    remote
        .push(&[name], Some(&mut options))
        .with_context(|| "Failed to push the repository")?;

    Ok(())
}

pub fn list_feature_branches() -> Result<Vec<String>> {
    let repo = Repository::open(".").with_context(|| "Failed to open the repository")?;
    let branches = repo
        .branches(Some(git2::BranchType::Local))
        .with_context(|| "Failed to get the branches")?
        .filter_map(|b| {
            match  b  {
               Ok((branch, _)) if matches!(branch.name(), Ok(Some(name)) if name.starts_with("feature/")) => Some(branch.name().unwrap().unwrap().to_string()) ,
               _ => None,
            }
        })
        .collect::<Vec<String>>();

    Ok(branches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone() {
        clone("https://github.com/phostann/host-template.git")
            .expect("Failed to clone git repository");
    }
}
