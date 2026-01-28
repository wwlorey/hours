use std::fs;

use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use clap::Args;

use crate::config::{Config, DataConfig, GitConfig, LicensureConfig};
use crate::data::model::HoursData;
use crate::data::store;
use crate::data::week;
use crate::git;
use crate::ui;

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

pub fn run(args: InitArgs, no_git: bool) -> Result<()> {
    let config_path = Config::config_path();
    if config_path.exists() {
        bail!(
            "Already initialized. Config exists at {}",
            config_path.display()
        );
    }

    let (data_dir, remote_url, start_date) = if args.non_interactive {
        let data_dir = args
            .data_dir
            .ok_or_else(|| anyhow::anyhow!("--data-dir is required in non-interactive mode"))?;
        let remote = args
            .remote
            .ok_or_else(|| anyhow::anyhow!("--remote is required in non-interactive mode"))?;
        let start_str = args
            .start_date
            .ok_or_else(|| anyhow::anyhow!("--start-date is required in non-interactive mode"))?;
        let start = NaiveDate::parse_from_str(&start_str, "%Y-%m-%d")
            .with_context(|| format!("Invalid date format: {start_str}"))?;
        if !week::is_tuesday(start) {
            bail!("Start date must be a Tuesday, got {start}");
        }
        (data_dir, remote, start)
    } else {
        let data_dir = ui::input_text("Data directory", Some("~/Sync/.hours"))?
            .ok_or_else(|| anyhow::anyhow!("Cancelled"))?;

        let remote =
            ui::input_text("Git remote URL", None)?.ok_or_else(|| anyhow::anyhow!("Cancelled"))?;

        let start = ui::input_date("Licensure start date", true)?
            .ok_or_else(|| anyhow::anyhow!("Cancelled"))?;

        (data_dir, remote, start)
    };

    let data_dir_expanded = shellexpand::tilde(&data_dir).into_owned();

    let config = Config {
        data: DataConfig {
            directory: data_dir,
        },
        git: GitConfig {
            remote: "origin".to_string(),
            auto_push: true,
        },
        licensure: LicensureConfig {
            start_date,
            total_hours_target: 3000,
            direct_hours_target: 1200,
            min_months: 24,
            min_weekly_average: 15.0,
        },
    };

    config.save(&config_path)?;
    println!("Config saved to {}", config_path.display());

    let data_path = std::path::PathBuf::from(&data_dir_expanded);
    fs::create_dir_all(&data_path)
        .with_context(|| format!("Failed to create data directory {}", data_path.display()))?;

    let data_file = data_path.join("hours.json");
    let data = HoursData::new();
    store::save(&data_file, &data)?;
    println!("Created {}", data_file.display());

    git::git_init_and_commit(&data_path, &config.git, &remote_url, no_git)?;

    println!("Initialized hours tracking.");
    Ok(())
}
