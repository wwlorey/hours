use clap::Args;

#[derive(Args)]
pub struct ExportArgs {
    #[arg(long, help = "Override output file path")]
    pub output: Option<String>,

    #[arg(long, help = "Open the PDF after generation")]
    pub open: bool,
}

pub fn run(_args: ExportArgs, _no_git: bool) -> anyhow::Result<()> {
    todo!()
}
