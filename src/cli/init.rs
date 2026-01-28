use clap::Args;

#[derive(Args)]
pub struct InitArgs {
    #[arg(long, help = "Path to data directory")]
    pub data_dir: Option<String>,

    #[arg(long, help = "Git remote URL")]
    pub remote: Option<String>,

    #[arg(long, help = "Licensure start date (YYYY-MM-DD, must be a Tuesday)")]
    pub start_date: Option<String>,

    #[arg(long, help = "Run without interactive prompts")]
    pub non_interactive: bool,
}

pub fn run(_args: InitArgs, _no_git: bool) -> anyhow::Result<()> {
    todo!()
}
