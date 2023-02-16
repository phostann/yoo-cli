use std::{
    io::{BufReader, Read},
    process::{Command, Stdio},
    vec,
};

pub fn clone(repo: &str) {
    exec_git_command(vec![
        "clone",
        repo,
        ".", // clone into the current directory
    ])
}

// git init command
pub fn init() {
    exec_git_command(vec!["init"])
}

fn exec_git_command(args: Vec<&str>) {
    let child = Command::new("git")
        .args(args.clone())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to execute process: git {}", args.join(" ")));

    let mut reader = BufReader::new(child.stdout.expect("Failed to read from process"));

    let mut buffer = String::new();

    reader
        .read_to_string(&mut buffer)
        .expect("Failed to read from process");

    println!("{}", buffer);
}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone() {
        clone("https://github.com/phostann/host-template.git");
    }
}
