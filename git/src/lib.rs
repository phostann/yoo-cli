use std::env;

use anyhow::{Context, Result};
use git2::{Cred, CredentialType, Repository, StatusOptions};

pub struct GitRepo {
    repo: Repository,
    username: String,
    password: String,
}

pub fn open_repo(path: &str, username: String, password: String) -> Result<GitRepo> {
    let repo = Repository::open(path).with_context(|| "Failed to open the repository")?;
    Ok(GitRepo {
        repo,
        username,
        password,
    })
}

pub fn clone(repo: &str, path: &str, username: String, password: String) -> Result<GitRepo> {
    Repository::clone(repo, path).with_context(|| "Failed to clone git repository")?;
    open_repo(path, username, password)
}

impl GitRepo {
    pub fn delete_remote(&self) -> Result<()> {
        self.repo
            .remote_delete("origin")
            .with_context(|| "Failed to delete the remote")?;
        Ok(())
    }

    pub fn checkout_to_branch(&self, branch: &str) -> Result<()> {
        let head = self.repo.head().with_context(|| "Failed to get the head")?;
        let commit = head
            .peel_to_commit()
            .with_context(|| "Failed to get the commit")?;

        self.repo
            .branch(branch, &commit, false)
            .with_context(|| "Failed to create the branch")?;

        let (object, reference) = self
            .repo
            .revparse_ext(branch)
            .with_context(|| "Failed to get the object and reference")?;

        self.repo
            .checkout_tree(&object, None)
            .with_context(|| "Failed to checkout the tree")?;

        if let Some(gref) = reference {
            self.repo
                .set_head(
                    gref.name().ok_or_else(|| {
                        anyhow::Error::msg("Failed to get the name of the reference")
                    })?,
                )
                .with_context(|| "Failed to set the head")?;
        } else {
            self.repo
                .set_head_detached(object.id())
                .with_context(|| "Failed to set the head")?;
        }

        Ok(())
    }

    pub fn has_uncommitted_changes(&self) -> Result<bool> {
        let mut status_opts = StatusOptions::new();
        status_opts.include_ignored(false);
        let statuses = self
            .repo
            .statuses(Some(&mut status_opts))
            .with_context(|| "Failed to get the status of the repository")?;

        statuses.iter().for_each(|s| {
            tracing::info!(
                "status: {:?}, path: {:?}",
                s.status(),
                s.index_to_workdir().map(|p| p.new_file().path())
            );
        });

        if !statuses.is_empty() {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn push(&self, branch: &str) -> Result<()> {
        let mut remote = self
            .repo
            .find_remote("origin")
            .with_context(|| "Failed to push the repository")?;

        let branch = self
            .repo
            .find_branch(branch, git2::BranchType::Local)
            .with_context(|| "Failed to find the branch")?;

        let refs = branch.into_reference();

        let name = refs.name().with_context(|| "The reference name is none")?;

        let mut callbacks = git2::RemoteCallbacks::new();

        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            // tracing::info!("allowed types: {:?}", _allowed_types);

            if _allowed_types.contains(CredentialType::SSH_KEY) {
                Cred::ssh_key(
                    _username_from_url.unwrap(),
                    Some(std::path::Path::new(&format!(
                        "{}/.ssh/id_ed25519.pub",
                        env::var("HOME").unwrap()
                    ))),
                    std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
                    None,
                )
            } else {
                Cred::userpass_plaintext(&self.username, &self.password)
            }
        });

        let mut options = git2::PushOptions::new();

        options.remote_callbacks(callbacks);

        // push the code to master branch
        remote
            .push(&[name], Some(&mut options))
            .with_context(|| "Failed to push the code")?;

        Ok(())
    }

    pub fn list_branches(&self) -> Result<Vec<String>> {
        let branches = self
            .repo
            .branches(Some(git2::BranchType::Local))
            .with_context(|| "Failed to get the branches")?
            .filter_map(|b| match b {
                Ok((branch, _)) if branch.name().is_ok() && branch.name().unwrap().is_some() => {
                    Some(branch.name().unwrap().unwrap().to_string())
                }
                _ => None,
            })
            .collect::<Vec<String>>();

        Ok(branches)
    }

    pub fn set_remote(&self, repo_url: &str) -> Result<()> {
        self.repo
            .remote_set_url("origin", repo_url)
            .with_context(|| "Failed to set the remote")?;
        Ok(())
    }
}
