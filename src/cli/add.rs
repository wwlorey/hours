use anyhow::{bail, Context, Result};
use chrono::{Local, NaiveDate};
use clap::Args;

use crate::config::Config;
use crate::data::model::Category;
use crate::data::{store, week};
use crate::git;
use crate::ui;

#[derive(Args)]
pub struct AddArgs {
    #[arg(long, help = "Tuesday start date of the week (YYYY-MM-DD)")]
    pub week: Option<String>,

    #[arg(long, help = "Hour category")]
    pub category: Option<String>,

    #[arg(long, allow_hyphen_values = true, help = "Number of hours to add")]
    pub hours: Option<f64>,

    #[arg(long, help = "Run without interactive prompts")]
    pub non_interactive: bool,
}

pub fn run(args: AddArgs, no_git: bool) -> Result<()> {
    let config = Config::load()?;
    let data_file = config.data_file();
    let mut data = store::load(&data_file)?;

    let today = Local::now().date_naive();

    let (week_start, category, hours) = if args.non_interactive {
        let week_start = match &args.week {
            Some(w) => {
                let date = NaiveDate::parse_from_str(w, "%Y-%m-%d")
                    .with_context(|| format!("Invalid date format: {w}"))?;
                if !week::is_tuesday(date) {
                    bail!("Week start date must be a Tuesday, got {date}");
                }
                date
            }
            None => week::current_week(today).0,
        };

        let cat_str = args
            .category
            .ok_or_else(|| anyhow::anyhow!("--category is required in non-interactive mode"))?;
        let category: Category = cat_str.parse()?;

        let hours = args
            .hours
            .ok_or_else(|| anyhow::anyhow!("--hours is required in non-interactive mode"))?;
        if hours < 0.0 {
            bail!("Hours must be >= 0, got {hours}");
        }

        (week_start, category, hours)
    } else {
        let weeks = week::all_weeks(config.licensure.start_date, today);
        let (current_start, _) = week::current_week(today);

        let week_start = ui::select_week(&weeks, &data, current_start)?
            .ok_or_else(|| anyhow::anyhow!("Cancelled"))?;

        let category = ui::select_category()?.ok_or_else(|| anyhow::anyhow!("Cancelled"))?;

        let hours = ui::input_hours(&format!("Hours to add ({category})"), None)?
            .ok_or_else(|| anyhow::anyhow!("Cancelled"))?;

        (week_start, category, hours)
    };

    let (_, week_end) = week::week_containing(week_start);
    let entry = match data.weeks.iter_mut().find(|w| w.start == week_start) {
        Some(entry) => entry,
        None => {
            data.weeks
                .push(crate::data::model::WeekEntry::new(week_start, week_end));
            data.weeks.last_mut().unwrap()
        }
    };
    entry.add(category, hours);

    store::save(&data_file, &data)?;

    println!("Added {hours:.1} {category} hours for week of {week_start}",);

    let message = format!(
        "Add {} {} hours for week of {}",
        hours, category, week_start
    );
    git::git_sync(&config.data_dir(), &config.git, &message, no_git)?;

    Ok(())
}
