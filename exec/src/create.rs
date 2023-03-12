use anyhow::{Context, Result};
use console::{style, Emoji};
use inquire::{validator::Validation, Confirm, Select, Text};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::{
    collections::HashMap,
    env,
    fmt::Display,
    fs::{remove_dir_all, remove_file, DirEntry},
    io::Error,
};

use crate::{loading::loading, Cli};

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

// lazy to initialize the reqwest client
static REQUEST: Lazy<reqwest::blocking::Client> = Lazy::new(reqwest::blocking::Client::new);

// check the current directory and ask the user if they want to continue
pub(crate) fn create(cli: &Cli) -> Result<()> {
    // ask the user for the project name
    let project_name = Text::new("Please enter the project name:")
    .with_validator(|input: &str| {
        let pattern = Regex::new(r"^[A-Za-z0-9]+(-[A-Za-z0-9]+)*$")
            .with_context(|| "Failed to create a regex pattern to check the project name").unwrap();
        if pattern.is_match(input) {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Your project name is invalid it must match the regexp `^[A-Za-z0-9]+(-[A-Za-z0-9]+)*$`".into()))
        }
    })
    .prompt().with_context(|| "Failed to interact with the user")?;

    let project_description = Text::new("Please enter the project description:")
        .prompt()
        .with_context(|| "Failed to interact with the user")?;

    // get working directory
    let working_dir = env::current_dir().with_context(|| "Failed to get the current directory")?;

    // project dir
    let project_dir = working_dir.join(project_name.as_str());
    // project path
    let project_path = project_dir
        .to_str()
        .with_context(|| "Failed to convert the path to string")?;

    // if the project_dir is not exist, create it
    if !project_dir.exists() {
        std::fs::create_dir(&project_dir)
            .with_context(|| "Failed to create the project directory")?;
        tracing::info!("Successfully created the project directory")
    }

    // if the project_dir is not empty, ask the user if they want to continue
    let dir = project_dir
        .read_dir()
        .with_context(|| "Failed to read the current directory")?;

    let arr = dir.collect::<Vec<Result<DirEntry, Error>>>();
    // check if the current directory is empty
    if !arr.is_empty() {
        let ans = Confirm::new(
            format!(
                "{}  The directory {} is not empty. Do you want to cleanup and continue?",
                Emoji("⚠️", style("!!!").red().to_string().as_ref()),
                project_name
            )
            .as_str(),
        )
        .with_default(false)
        .prompt()
        .with_context(|| "Failed to interact with the user")?;

        if ans {
            for ele in arr {
                let path = ele.with_context(|| "Failed to unwrap the element")?.path();
                if path.is_dir() {
                    remove_dir_all(path).with_context(|| "Failed to remove the directory")?;
                } else {
                    remove_file(path).with_context(|| "Failed to remove the file")?;
                }
            }
            tracing::info!("Successfully cleaned up the directory {}", project_name);
        } else {
            return Err(anyhow::Error::msg("User canceled the operation"));
        }
    }

    let server_url = cli.server.as_deref().with_context(|| "SERVER is not set")?;

    let pb = loading("Fetching the templates")?;
    // show the project list
    let resp = REQUEST
        .get(format!("{}/templates", server_url))
        .send()
        .with_context(|| "Failed to get the templates")?;
    // sleep 10 seconds
    pb.finish_and_clear();

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
    let repo_options = resp
        .data
        .content
        .iter()
        .map(|x| x.name.as_str())
        .collect::<Vec<&str>>();

    let repo_ans: &str = Select::new("Please select a template:", repo_options)
        .prompt()
        .with_context(|| "Failed to interact with the user")?;

    let selection = resp
        .data
        .content
        .iter()
        .find(|x| x.name == repo_ans)
        .unwrap();

    // clone the repo
    let pb = loading("Cloning")?;
    let git_repo = git::clone(&selection.repo, project_path)?;
    pb.finish_and_clear();
    tracing::info!(
        "Successfully created the project based on the template: {} -- {}",
        selection.name,
        selection.brief
    );

    // remove the remote origin
    git_repo.delete_remote()?;
    tracing::info!("Successfully removed the remote origin");

    // create the new repo
    let pb = loading("Creating")?;
    let repo = create_new_repo(
        cli.gitlab_server.as_deref().unwrap(),
        cli.gitlab_token.as_deref().unwrap(),
        cli.gitlab_namespace_id.unwrap(),
        project_name.as_str(),
        &project_description,
    )?;
    pb.finish_and_clear();
    tracing::info!("Successfully created the new repo");
    tracing::info!("Repo: {}", repo);

    // add the remote origin
    git_repo.set_remote(repo.ssh_url_to_repo.as_str())?;
    tracing::info!(
        "Successfully added the remote origin: {}",
        repo.ssh_url_to_repo
    );

    // push the master branch to the remote origin
    let pb = loading("Pushing")?;
    git_repo.push("master")?;
    pb.finish_and_clear();
    tracing::info!("Successfully pushed the master branch to the remote origin");

    // create and checkout to dev branch
    git_repo.checkout_to_dev()?;
    tracing::info!("Successfully created and checked out to dev branch");

    // push the dev branch to the remote origin
    let pb = loading("Pushing")?;
    git_repo.push("dev")?;
    pb.finish_and_clear();
    tracing::info!("Successfully pushed the dev branch to the remote origin");

    Ok(())
    // use gitlab api to create a new repo and cache the info
}

#[derive(Debug, Deserialize)]
struct RepoResponse {
    id: u32,
    name: String,
    ssh_url_to_repo: String,
    web_url: String,
}

impl Display for RepoResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id: {}, name: {}, ssh_url_to_repo: {}, web_url: {}",
            self.id, self.name, self.ssh_url_to_repo, self.web_url
        )
    }
}

fn create_new_repo(
    gitlab_server: &str,
    gitlab_token: &str,
    gitlab_namespace_id: u32,
    name: &str,
    description: &str,
) -> Result<RepoResponse> {
    let mut map = HashMap::new();
    map.insert("name", name);
    map.insert("description", description);
    let namespace_id = gitlab_namespace_id.to_string();
    map.insert("namespace_id", namespace_id.as_str());
    map.insert("visibility", "internal");

    let res = REQUEST
        .post(format!("{}/api/v4/projects", gitlab_server))
        .header("PRIVATE-TOKEN", gitlab_token)
        .json(&map)
        .send()
        .with_context(|| "Failed to create the new repo")?;

    // check the status code
    if res.status() != 201 {
        // print the error message
        return Err(anyhow::Error::msg(format!(
            "Failed to create the new repo: {}",
            res.text()
                .with_context(|| "Failed to get the error message")?
        )));
    }

    let repo = res
        .json::<RepoResponse>()
        .with_context(|| "Failed to parse the response")?;

    Ok(repo)
}

// test
#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_create_project() {
        create_new_repo(
            "https://gitlab.com",
            "",
            64903429,
            "test",
            "test project",
        )
        .expect("Failed to create the new repo");
    }
}
