use anyhow::{Context, Result};
use console::{style, Emoji};
use inquire::{validator::Validation, Confirm, Select, Text};
use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fmt::Display,
    fs::{remove_dir_all, remove_file, DirEntry, File, OpenOptions},
    io::{BufReader, BufWriter, Error},
};

use crate::{loading::loading, Cli, CACHE_DIR, CACHE_FILE, REQUEST};

#[derive(Debug, Deserialize)]
struct Response<T> {
    code: i32,
    // msg: String,
    data: T,
}

#[derive(Debug, Deserialize)]
struct PageData<T> {
    content: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct Template {
    name: String,
    repo: String,
    brief: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Project {
    id: i32,
    name: String,
    repo: String,
    repo_id: i32,
    description: String,
}

// check the current directory and ask the user if they want to continue
pub(crate) fn create(cli: &mut Cli) -> Result<()> {
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

    // ask the user for the category or the project
    let category = Text::new("Please enter the category of the project:")
        .prompt().with_context(|| "Failed to interact with the user")?;
    let tags: String = Text::new("Please enter the tags of the project, use `,` to separate them:")
        .prompt().with_context(|| "Failed to interact with the user")?;

    let build_cmd = Text::new("Please enter the build command of the project:")
        .with_default("yarn build")
        .prompt().with_context(|| "Failed to interact with the user")?;

    let dist = Text::new("Please enter the dist of the project:")
        .with_default("build")
        .prompt().with_context(|| "Failed to interact with the user")?;

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
        cli.repo_name = Some(project_name.clone());
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

    tracing::debug!("server_url: {}", server_url);

    let pb = loading("Fetching the templates")?;
    // show the project list
    let resp = REQUEST
        .get(format!("{}/v1/templates", server_url))
        .send()
        .with_context(|| "Failed to get the templates")?;
    // sleep 10 seconds
    pb.finish_and_clear();

    if resp.status() != 200 {
        return Err(anyhow::Error::msg("Failed to get the templates"));
    }

    let resp = resp
        .json::<Response<PageData<Template>>>()
        .with_context(|| "Failed to parse the response")?;

    if resp.code != 0 {
        return Err(anyhow::Error::msg("Failed to get the templates"));
    }
    if resp.data.content.is_empty() {
        return Err(anyhow::Error::msg("No templates found"));
    }

    // show the options of the templates
    let repo_options: Vec<String> = resp
        .data
        .content
        .iter()
        .map(|x| format!("{} -- {}", x.name, x.brief))
        .collect();

    let repo_ans: String = Select::new("Please select a template:", repo_options)
        .prompt()
        .with_context(|| "Failed to interact with the user")?;

    let selection = resp
        .data
        .content
        .iter()
        .find(|x| format!("{} -- {}", x.name, x.brief) == repo_ans)
        .with_context(|| "Failed to find the template")?;

    // clone the repo
    let pb = loading("Cloning")?;
    let mut git_repo = git::clone(
        &selection.repo,
        project_path,
    )?;
    pb.finish_and_clear();
    tracing::info!(
        "Successfully created the project based on the template: {} -- {}",
        selection.name,
        selection.brief
    );

    cli.repo_name = Some(project_name.clone());

    // remove the remote origin
    git_repo.delete_remote()?;
    tracing::info!("Successfully removed the remote origin");

    // create the new repo
    let pb = loading("Creating")?;
    let repo = create_new_repo(cli, project_name.as_str(), &project_description)?;
    cli.repo_id = Some(repo.id);
    pb.finish_and_clear();
    tracing::info!("Successfully created the new repo");
    tracing::info!("Repo: {}", repo);

    // add the remote origin
    git_repo.set_remote(repo.ssh_url_to_repo.as_str())?;
    tracing::info!(
        "Successfully added the remote origin: {}",
        repo.ssh_url_to_repo
    );

    // change working dir
    git_repo.change_working_dir(Some(project_path.to_string()))?;

    // push the master branch to the remote origin
    let pb = loading("Pushing")?;
    git_repo.push("master")?;
    pb.finish_and_clear();
    tracing::info!("Successfully pushed the master branch to the remote origin");

    // create and checkout to dev branch
    git_repo.checkout_to_branch("dev")?;
    tracing::info!("Successfully created and checked out to dev branch");

    // push the dev branch to the remote origin
    let pb = loading("Pushing")?;
    git_repo.push("dev")?;
    pb.finish_and_clear();
    tracing::info!("Successfully pushed the dev branch to the remote origin");

    // reset the working dir
    git_repo.change_working_dir(None)?;

    // register the project to the server
    let pb = loading("Registering")?;

    let tags = tags.split(',').map(|x| x.trim()).collect::<Vec<&str>>();

    let payload = NewProject {
        name: project_name.trim(),
        description: project_description.trim(),
        pid: repo.id,
        category: category.trim(),
        tags,
        ssh_url: repo.ssh_url_to_repo.as_ref(),
        http_url: repo.http_url_to_repo.as_ref(),
        web_url: repo.web_url.as_ref(),
        build_cmd: build_cmd.trim(),
        dist: dist.trim(),
    };

    register_project(
        cli,
        &payload,
    )?;

    pb.finish_and_clear();

    tracing::info!("Successfully registered the project to the server");

    tracing::info!("Now everything is ready. You can start to work on your project. keep coding!");

    Ok(())
    // use gitlab api to create a new repo and cache the info
}

#[derive(Debug, Deserialize)]
struct RepoResponse {
    id: i32,
    name: String,
    ssh_url_to_repo: String,
    http_url_to_repo: String,
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

fn create_new_repo(cli: &Cli, name: &str, description: &str) -> Result<RepoResponse> {
    let mut map = HashMap::new();
    map.insert("name", name);
    map.insert("description", description);
    let namespace_id = cli.gitlab_namespace_id.unwrap().to_string();
    map.insert("namespace_id", namespace_id.as_str());
    map.insert("visibility", "internal");

    let res = REQUEST
        .post(format!(
            "{}/api/v4/projects",
            cli.gitlab_server.clone().unwrap()
        ))
        .header("PRIVATE-TOKEN", cli.gitlab_token.clone().unwrap())
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

#[derive(Debug, Serialize)]
struct NewProject<'a> {
    name: &'a str,
    category: &'a str,
    tags: Vec<&'a str>,
    ssh_url: &'a str,
    http_url: &'a str,
    web_url: &'a str,
    build_cmd: &'a str,
    dist: &'a str,
    pid: i32,
    description: &'a str,
}

fn register_project(
    cli: &Cli,
    payload: &NewProject,
) -> Result<()> {
    let mut authorization = match read_authorization() {
        Ok(auth) => auth,
        Err(_) => {
            let auth = login(
                cli.server.as_ref().unwrap(),
                cli.server_email.as_ref().unwrap(),
                cli.server_password.as_ref().unwrap(),
            )?;
            write_authorization(&auth)?;
            auth
        }
    };

    let resp = register_project_to_server(cli, &payload, &authorization)?;

    match resp.status() {
        StatusCode::OK => {
            Ok(())
        }
        StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED => {
            authorization = login(
                cli.server.as_ref().unwrap(),
                cli.server_email.as_ref().unwrap(),
                cli.server_password.as_ref().unwrap(),
            )?;

            write_authorization(&authorization)?;

            let resp = register_project_to_server(cli, &payload, &authorization)?;

            if resp.status() != StatusCode::OK {
                return Err(anyhow::Error::msg(
                    "Failed to register the project".to_string(),
                ));
            }

            Ok(())
        }
        _ => Err(anyhow::Error::msg(
            "Failed to register the project".to_string(),
        )),
    }
}

fn register_project_to_server(cli: &Cli, payload: &&NewProject, authorization: &String) -> Result<reqwest::blocking::Response> {
    REQUEST
        .post(format!("{}/v1/projects", cli.server.as_ref().unwrap()))
        .header("Authorization", authorization)
        .json(&payload)
        .send()
        .with_context(|| "Failed to register the project")
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AuthBody {
    access_token: String,
    refresh_token: String,
    token_type: String,
}

fn login(server_url: &str, email: &str, password: &str) -> Result<String> {
    let mut map = HashMap::new();
    map.insert("email", email);
    map.insert("password", password);

    let resp = REQUEST
        .post(format!("{}/v1/users/login", server_url))
        .json(&map)
        .send()?;

    // check the status code
    if resp.status() != 200 {
        return Err(anyhow::Error::msg("Failed to login".to_string()));
    }

    let auth_resp = resp
        .json::<Response<AuthBody>>()
        .with_context(|| "Failed to parse the response")?;

    if auth_resp.code != 0 {
        return Err(anyhow::Error::msg("Failed to login".to_string()));
    }

    Ok(format!(
        "{} {}",
        auth_resp.data.token_type, auth_resp.data.access_token
    ))
}

fn read_authorization() -> Result<String> {
    // get home dir
    let home_dir = dirs::home_dir().with_context(|| "Failed to get the home dir")?;
    let cache_file = home_dir.join(format!("{}/{}", CACHE_DIR, CACHE_FILE));
    // read cache json file
    let file = File::open(cache_file)?;
    let reader = BufReader::new(file);
    let cache: HashMap<String, String> = serde_json::from_reader(reader)?;
    let token = cache.get("authorization");

    if let Some(token) = token {
        Ok(token.to_string())
    } else {
        Err(anyhow::Error::msg("Failed to read the token"))
    }
}

fn write_authorization(authorization: &str) -> Result<()> {
    let home_dir = dirs::home_dir().with_context(|| "Failed to get the home dir")?;
    let cache_file = home_dir.join(format!("{}/{}", CACHE_DIR, CACHE_FILE));

    // read cache json file
    let mut cache: HashMap<String, String> = HashMap::new();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(cache_file)?;

    if file.metadata()?.len() > 0 {
        let reader = BufReader::new(&file);
        cache = serde_json::from_reader(reader)?;
    }

    cache.insert("authorization".to_string(), authorization.to_string());

    // write cache json file
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &cache)?;
    Ok(())
}
