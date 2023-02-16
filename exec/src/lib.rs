use anyhow::Result;
use clap::{Parser, Subcommand};

mod create;

#[derive(Parser)]
#[command(name = "yoo")]
#[command(author = "phos")]
#[command(version = "0.1.0")]
#[command(
    about = "An awesome CLI tool for frontend developers",
    long_about = None 
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// create a new project
    Create {
    },
    /// Upload the project
    Submit {
        #[arg(short, long)]
        test: bool,
    },
}

pub fn init() -> Result<()> {

    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Create{}) => {
            create::create();
        }
        Some(Commands::Submit { test }) => {
            println!("test: {}", test);
        }
        None => {
            println!("Nothing to do");
        }
    }

    Ok(())
}


// test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

}
