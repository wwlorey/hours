use clap::Args;

#[derive(Args)]
pub struct EditArgs {
    #[arg(long, help = "Tuesday start date of the week (YYYY-MM-DD)")]
    pub week: Option<String>,

    #[arg(long, help = "Individual supervision hours")]
    pub individual_supervision: Option<f64>,

    #[arg(long, help = "Group supervision hours")]
    pub group_supervision: Option<f64>,

    #[arg(long, help = "Direct client contact hours")]
    pub direct: Option<f64>,

    #[arg(long, help = "Indirect hours")]
    pub indirect: Option<f64>,

    #[arg(long, help = "Run without interactive prompts")]
    pub non_interactive: bool,
}

pub fn run(_args: EditArgs, _no_git: bool) -> anyhow::Result<()> {
    todo!()
}
