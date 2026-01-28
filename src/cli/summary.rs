use anyhow::Result;
use chrono::Local;
use clap::Args;

use crate::config::Config;
use crate::data::store;
use crate::data::week;

#[derive(Args)]
pub struct SummaryArgs {
    #[arg(long, help = "Output as JSON")]
    pub json: bool,
}

fn months_between(start: chrono::NaiveDate, end: chrono::NaiveDate) -> u32 {
    if end < start {
        return 0;
    }
    let year_diff = end.year() - start.year();
    let month_diff = end.month() as i32 - start.month() as i32;
    let mut months = year_diff * 12 + month_diff;
    if end.day() < start.day() {
        months -= 1;
    }
    months.max(0) as u32
}

use chrono::Datelike;

pub fn run(args: SummaryArgs) -> Result<()> {
    let config = Config::load()?;
    let data_file = config.data_file();
    let data = store::load(&data_file)?;

    let today = Local::now().date_naive();
    let start_date = config.licensure.start_date;

    let total_hours: f64 = data.weeks.iter().map(|w| w.total()).sum();
    let direct_hours: f64 = data.weeks.iter().map(|w| w.direct).sum();

    let months = months_between(start_date, today);

    let (current_week_start, _) = week::current_week(today);
    let weeks_elapsed = if current_week_start >= start_date {
        ((current_week_start - start_date).num_days() / 7) + 1
    } else {
        1
    };

    let weekly_average = if weeks_elapsed > 0 {
        total_hours / weeks_elapsed as f64
    } else {
        0.0
    };

    let total_target = config.licensure.total_hours_target;
    let direct_target = config.licensure.direct_hours_target;
    let min_months = config.licensure.min_months;
    let min_weekly_avg = config.licensure.min_weekly_average;

    let total_pct = if total_target > 0 {
        total_hours / total_target as f64 * 100.0
    } else {
        0.0
    };
    let direct_pct = if direct_target > 0 {
        direct_hours / direct_target as f64 * 100.0
    } else {
        0.0
    };
    let months_pct = if min_months > 0 {
        months as f64 / min_months as f64 * 100.0
    } else {
        0.0
    };
    let avg_pct = if min_weekly_avg > 0.0 {
        weekly_average / min_weekly_avg * 100.0
    } else {
        0.0
    };

    let weeks_logged = data.weeks.iter().filter(|w| w.total() > 0.0).count();

    if args.json {
        let mut json = serde_json::json!({
            "total_hours": {
                "current": round1(total_hours),
                "target": total_target,
                "percentage": round1(total_pct),
            },
            "direct_hours": {
                "current": round1(direct_hours),
                "target": direct_target,
                "percentage": round1(direct_pct),
            },
            "months": {
                "current": months,
                "target": min_months,
                "percentage": round1(months_pct),
            },
            "weekly_average": {
                "current": round1(weekly_average),
                "target": min_weekly_avg,
                "percentage": round1(avg_pct),
            },
            "weeks_logged": weeks_logged,
            "start_date": start_date.format("%Y-%m-%d").to_string(),
        });

        if let Some(last) = data.weeks.last() {
            json["latest_week_start"] =
                serde_json::Value::String(last.start.format("%Y-%m-%d").to_string());
            json["latest_week_end"] =
                serde_json::Value::String(last.end.format("%Y-%m-%d").to_string());
        }

        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("Licensure Progress");
        println!("{}", "═".repeat(50));
        println!();
        println!(
            "Total supervised hours: {:>8.1} / {:<6} ({:>5.1}%)",
            total_hours, total_target, total_pct
        );
        println!(
            "Direct client hours:   {:>8.1} / {:<6} ({:>5.1}%)",
            direct_hours, direct_target, direct_pct
        );
        println!(
            "Months of experience:  {:>8}   / {:>4}   ({:>5.1}%)",
            months, min_months, months_pct
        );
        println!(
            "Weekly average:        {:>8.1} / {:>6.1} ({:>5.1}%)",
            weekly_average, min_weekly_avg, avg_pct
        );
        println!();
        println!("Weeks logged: {weeks_logged}");

        if !data.weeks.is_empty() {
            let first = &data.weeks[0];
            let last = data.weeks.last().unwrap();
            println!(
                "Date range: {} – {}",
                first.start.format("%b %d, %Y"),
                last.end.format("%b %d, %Y")
            );
        }
    }

    Ok(())
}

fn round1(val: f64) -> f64 {
    (val * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_months_between_same_date() {
        assert_eq!(months_between(date(2025, 1, 28), date(2025, 1, 28)), 0);
    }

    #[test]
    fn test_months_between_one_month() {
        assert_eq!(months_between(date(2025, 1, 28), date(2025, 2, 28)), 1);
    }

    #[test]
    fn test_months_between_partial_month() {
        assert_eq!(months_between(date(2025, 1, 28), date(2025, 2, 27)), 0);
    }

    #[test]
    fn test_months_between_several_months() {
        assert_eq!(months_between(date(2025, 1, 28), date(2025, 6, 28)), 5);
    }

    #[test]
    fn test_months_between_across_years() {
        assert_eq!(months_between(date(2025, 1, 28), date(2027, 1, 28)), 24);
    }

    #[test]
    fn test_months_between_end_before_start() {
        assert_eq!(months_between(date(2025, 6, 1), date(2025, 1, 1)), 0);
    }

    #[test]
    fn test_round1() {
        assert!((round1(8.233) - 8.2).abs() < f64::EPSILON);
        assert!((round1(102.75) - 102.8).abs() < f64::EPSILON);
        assert!((round1(0.0) - 0.0).abs() < f64::EPSILON);
    }
}
