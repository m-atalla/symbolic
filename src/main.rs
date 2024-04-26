use std::process;

use clap::{Parser, Subcommand, Args};
/// Configuration files symbolic linking utility
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands, 
    #[arg(default_value="$HOME")]
    home: Option<String>,
}


#[derive(Subcommand)]
enum Commands {
    /// links file using symbolic syntax `<SOURCE> -> <TARGET>`
    Link { sym: String },
    /// update symbolic links for the provided `.sym` file path. (default=".")
    Update(PathArg),
    /// breaks target links for the provided `.sym` file path. (default=".")
    Break(PathArg) 
}


#[derive(Args)]
struct PathArg {
    path: Option<String>
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Link{..} => { 
            unimplemented!()
        },
        Commands::Update(_arg) => { 
            if let Err(err) = symbolic::run() {
                eprintln!("{}", err);
                process::exit(1);
            }
        },
        Commands::Break(_arg) => { 
            unimplemented!()
        }
    }
}
