mod add;
mod edit;
mod export;
mod init;
mod list;
mod summary;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hours", about = "Track counseling licensure hours")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, global = true, help = "Disable git operations")]
    pub no_git: bool,
}

#[derive(Subcommand)]
pub enum Command {
    Init(init::InitArgs),
    Add(add::AddArgs),
    Edit(edit::EditArgs),
    List(list::ListArgs),
    Summary(summary::SummaryArgs),
    Export(export::ExportArgs),
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Init(args) => init::run(args, cli.no_git),
        Command::Add(args) => add::run(args, cli.no_git),
        Command::Edit(args) => edit::run(args, cli.no_git),
        Command::List(args) => list::run(args),
        Command::Summary(args) => summary::run(args),
        Command::Export(args) => export::run(args, cli.no_git),
    }
}
