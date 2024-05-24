use std::process;

use clap::{Args, Parser, Subcommand};
/// Configuration files symbolic linking utility
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(default_value = "$HOME")]
    home: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// links file from `<SOURCE> to -> <TARGET>`
    Link { source: String, target: String },
    /// update symbolic links for the provided `.sym` file path. (default=".")
    #[clap(visible_alias("up"))]
    Update(PathArg),
    /// breaks target links for the provided `.sym` file path. (default=".")
    Break(PathArg),
}

#[derive(Args, Debug)]
struct PathArg {
    path: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Link { source, target } => {
            if let Err(err) = symbolic::link_up(source, target) {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
        Commands::Update(_arg) => {
            if let Err(err) = symbolic::run() {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
        Commands::Break(_arg) => {
            println!("path: {:?}", _arg);
            unimplemented!()
        }
    }
}
