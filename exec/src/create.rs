use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use serde::Deserialize;
use std::{
    env,
    fs::{remove_dir_all, remove_file, DirEntry},
    io::Error,
    path::PathBuf,
};
use console::{Emoji, style};

#[derive(Debug, Deserialize)]
struct Response {
    code: i32,
    msg: String,
    data: ResponseData,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    content: Vec<Template>,
}

#[derive(Debug, Deserialize)]
struct Template {
    id: i32,
    name: String,
    repo: String,
    brief: String, }

// check the current directory and ask the user if they want to continue
pub(crate) fn create() {
    // get working directory
    let working_dir = env::current_dir().expect("Failed to get working directory");

    let dir = working_dir
        .read_dir()
        .expect("Failed to read the current directory");

    let arr = dir.collect::<Vec<Result<DirEntry, Error>>>();
    // check if the current directory is empty
    if !arr.is_empty() {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{}  The current directory is not empty. Do you want to cleanup and continue?", Emoji("⚠️", style("!!!").red().to_string().as_ref())))
            .default(false)
            .wait_for_newline(true)
            .interact()
            .expect("Failed to interact with the user")
        {
            for ele in arr {
                let path = ele.expect("Failed to unwrap the element");
                let path = path.path();
                if path.is_dir() {
                    remove_dir_all(path).expect("Failed to remove the directory");
                } else {
                    remove_file(path).expect("Failed to remove the file");
                }
            }
        } else {
            log::error!("Aborted by the user");
            std::process::exit(0);
        }
    }
    // show the project list
    let resp = reqwest::blocking::get("http://192.168.31.120:8989/templates")
        .expect("Failed to get the templates");
    let resp = resp
        .json::<Response>()
        .expect("Failed to parse the response");
    if resp.code != 0 {
        log::error!("Failed to get the templates: {}", resp.msg);
        std::process::exit(0);
    }
    if resp.data.content.is_empty() {
        log::error!("No templates found");
        std::process::exit(0);
    }
    // show the options of the templates

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(
            resp.data
                .content
                .iter()
                .map(|x| format!("{} -- {}", x.name, x.brief))
                .collect::<Vec<String>>()
                .as_slice(),
        )
        .default(0)
        .interact()
        .expect("Failed to interact with the user");

    let seclection = &resp.data.content[selection];

    // clone the repo
    log::info!("Cloning the the {} into current directory", seclection.repo);
    git::clone(&seclection.repo);

    // remove the .git directory
    let git_dir = PathBuf::from(".git");
    if git_dir.exists() && git_dir.is_dir() {
        remove_dir_all(git_dir).expect("Failed to remove the .git directory");
    }

    // init the git repo
    log::info!("Initializing the git repository");
    git::init();
}
