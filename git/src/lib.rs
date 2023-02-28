use anyhow::{Context, Result};
use git2::Repository;

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
    Repository::open(".").with_context(|| "")?;
    Ok(true)
}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone() {
        clone("https://github.com/phostann/host-template.git").expect("Failed to clone git repository");
    }
}
