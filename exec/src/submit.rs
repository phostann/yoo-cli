use anyhow::{Result, Ok};


pub(crate) fn submit() -> Result<()>{
    // prepare to submit
    prepare();
    Ok(())
}

fn prepare () {
   // 1. check if the current directory is a valid git project
   git::is_git_project().expect("Failed to check if the current directory is a valid git project");
}
