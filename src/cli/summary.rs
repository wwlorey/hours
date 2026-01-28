use clap::Args;

#[derive(Args)]
pub struct SummaryArgs {
    #[arg(long, help = "Output as JSON")]
    pub json: bool,
}

pub fn run(_args: SummaryArgs) -> anyhow::Result<()> {
    todo!()
}
