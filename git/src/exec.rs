use std::process::Command;
use anyhow::{Context, Error, Result};

pub fn exec_git_command(args: &Vec<&str>, working_dir: Option<&str>) -> Result<String> {
    let output = Command::new("git").args(args)
        .current_dir(if let Some(working_dir) = working_dir { working_dir } else { "." })
        .output()
        .with_context(|| "Failed to execute the git command")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(Error::msg(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

// test
#[cfg(test)]
mod test {
    use crate::exec::exec_git_command;

    #[test]
    fn test_clone() {
        exec_git_command(&vec!["clone", "ssh://git@192.168.31.162:222/yoo/test-ssh.git"], None).unwrap();
    }
}