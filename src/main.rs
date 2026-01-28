mod cli;
#[allow(dead_code)]
mod config;
#[allow(dead_code)]
mod data;
#[allow(dead_code)]
mod git;
#[allow(dead_code)]
mod pdf;
#[allow(dead_code)]
mod ui;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    if let Err(e) = cli::run(cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
