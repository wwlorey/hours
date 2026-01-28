use anyhow::{bail, Context, Result};
use chrono::{Local, NaiveDate};
use clap::Args;

use crate::config::Config;
use crate::data::model::{Category, WeekEntry};
use crate::data::{store, week};
use crate::git;
use crate::ui;

#[derive(Args)]
pub struct EditArgs {
    #[arg(long, help = "Tuesday start date of the week (YYYY-MM-DD)")]
    pub week: Option<String>,

    #[arg(
        long,
        allow_hyphen_values = true,
        help = "Individual supervision hours"
    )]
    pub individual_supervision: Option<f64>,

    #[arg(long, allow_hyphen_values = true, help = "Group supervision hours")]
    pub group_supervision: Option<f64>,

    #[arg(long, allow_hyphen_values = true, help = "Direct client contact hours")]
    pub direct: Option<f64>,

    #[arg(long, allow_hyphen_values = true, help = "Indirect hours")]
    pub indirect: Option<f64>,

    #[arg(long, help = "Run without interactive prompts")]
    pub non_interactive: bool,
}

pub fn run(args: EditArgs, no_git: bool) -> Result<()> {
    let config = Config::load()?;
    let data_file = config.data_file();
    let mut data = store::load(&data_file)?;

    let today = Local::now().date_naive();

    if args.non_interactive {
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

        let (_, week_end) = week::week_containing(week_start);
        let entry = match data.weeks.iter_mut().find(|w| w.start == week_start) {
            Some(entry) => entry,
            None => {
                data.weeks.push(WeekEntry::new(week_start, week_end));
                data.weeks.last_mut().unwrap()
            }
        };

        if let Some(val) = args.individual_supervision {
            if val < 0.0 {
                bail!("Hours must be >= 0");
            }
            entry.set(Category::IndividualSupervision, val);
        }
        if let Some(val) = args.group_supervision {
            if val < 0.0 {
                bail!("Hours must be >= 0");
            }
            entry.set(Category::GroupSupervision, val);
        }
        if let Some(val) = args.direct {
            if val < 0.0 {
                bail!("Hours must be >= 0");
            }
            entry.set(Category::Direct, val);
        }
        if let Some(val) = args.indirect {
            if val < 0.0 {
                bail!("Hours must be >= 0");
            }
            entry.set(Category::Indirect, val);
        }

        store::save(&data_file, &data)?;
        println!("Edited hours for week of {week_start}");

        let message = format!("Edit hours for week of {week_start}");
        git::git_sync(&config.data_dir(), &config.git, &message, no_git)?;
    } else {
        let weeks = week::all_weeks(config.licensure.start_date, today);
        let (current_start, _) = week::current_week(today);

        let week_start = ui::select_week(&weeks, &data, current_start)?
            .ok_or_else(|| anyhow::anyhow!("Cancelled"))?;

        let (_, week_end) = week::week_containing(week_start);
        let entry = match data.weeks.iter_mut().find(|w| w.start == week_start) {
            Some(entry) => entry,
            None => {
                data.weeks.push(WeekEntry::new(week_start, week_end));
                data.weeks.last_mut().unwrap()
            }
        };

        for cat in Category::ALL {
            let current = entry.get(cat);
            let prompt = cat.long_name().to_string();
            if let Some(val) = ui::input_hours(&prompt, Some(current))? {
                entry.set(cat, val);
            }
        }

        store::save(&data_file, &data)?;
        println!("Edited hours for week of {week_start}");

        let message = format!("Edit hours for week of {week_start}");
        git::git_sync(&config.data_dir(), &config.git, &message, no_git)?;
    }

    Ok(())
}
