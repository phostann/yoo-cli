use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod create;
mod submit;

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

    #[arg(short,long, default_value_t = false)]
    debug: bool
}

#[derive(Subcommand)]
enum Commands {
    /// create a new project
    Create {
    },
    /// Upload the project
    Submit {
    },
}

pub fn init() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    let level = if cli.debug { Level::DEBUG } else { Level::INFO };

    let subscriber = FmtSubscriber::builder()
    .with_max_level(level)
    .finish();

    tracing::subscriber::set_global_default(subscriber)
    .with_context(|| format!("Failed to set global default subscriber" ))?;

    match cli.command {
        Some(Commands::Create{}) => {
            return create::create();
        }
        Some(Commands::Submit { }) => {
            return submit::submit();
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
