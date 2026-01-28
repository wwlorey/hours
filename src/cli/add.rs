use clap::Args;

#[derive(Args)]
pub struct AddArgs {
    #[arg(long, help = "Tuesday start date of the week (YYYY-MM-DD)")]
    pub week: Option<String>,

    #[arg(long, help = "Hour category")]
    pub category: Option<String>,

    #[arg(long, help = "Number of hours to add")]
    pub hours: Option<f64>,

    #[arg(long, help = "Run without interactive prompts")]
    pub non_interactive: bool,
}

pub fn run(_args: AddArgs, _no_git: bool) -> anyhow::Result<()> {
    todo!()
}
