use anyhow::Result;
use chrono::Local;
use clap::Args;

use crate::config::Config;
use crate::data::store;
use crate::pdf;

#[derive(Args)]
pub struct ExportArgs {
    #[arg(long, help = "Override output file path")]
    pub output: Option<String>,

    #[arg(long, help = "Open the PDF after generation")]
    pub open: bool,
}

pub fn run(args: ExportArgs, _no_git: bool) -> Result<()> {
    let config = Config::load()?;
    let data_file = config.data_file();
    let data = store::load(&data_file)?;

    let today = Local::now().date_naive();
    let output_path = match &args.output {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let exports_dir = config.data_dir().join("exports");
            std::fs::create_dir_all(&exports_dir)?;
            exports_dir.join(format!("hours-report-{}.pdf", today.format("%Y-%m-%d")))
        }
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    pdf::generate_report(&data, &config.licensure, &output_path)?;

    println!("Report saved to {}", output_path.display());

    if args.open {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&output_path)
                .spawn()?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&output_path)
                .spawn()?;
        }
    }

    Ok(())
}
