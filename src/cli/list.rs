use clap::Args;

#[derive(Args)]
pub struct ListArgs {
    #[arg(long, help = "Output as JSON")]
    pub json: bool,

    #[arg(long, help = "Show only the last N weeks")]
    pub last: Option<usize>,
}

pub fn run(_args: ListArgs) -> anyhow::Result<()> {
    todo!()
}
