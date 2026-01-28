use anyhow::Result;
use clap::Args;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, Table};

use crate::config::Config;
use crate::data::model::Category;
use crate::data::store;

#[derive(Args)]
pub struct ListArgs {
    #[arg(long, help = "Output as JSON")]
    pub json: bool,

    #[arg(long, help = "Show only the last N weeks")]
    pub last: Option<usize>,
}

pub fn run(args: ListArgs) -> Result<()> {
    let config = Config::load()?;
    let data_file = config.data_file();
    let data = store::load(&data_file)?;

    if data.weeks.is_empty() {
        if args.json {
            println!("[]");
        } else {
            println!("No hours logged yet. Use `hours add` to start tracking.");
        }
        return Ok(());
    }

    let weeks = if let Some(n) = args.last {
        let len = data.weeks.len();
        if n >= len {
            &data.weeks[..]
        } else {
            &data.weeks[len - n..]
        }
    } else {
        &data.weeks[..]
    };

    if args.json {
        let json_weeks: Vec<serde_json::Value> = weeks
            .iter()
            .map(|w| {
                serde_json::json!({
                    "start": w.start.format("%Y-%m-%d").to_string(),
                    "end": w.end.format("%Y-%m-%d").to_string(),
                    "individual_supervision": w.individual_supervision,
                    "group_supervision": w.group_supervision,
                    "direct": w.direct,
                    "indirect": w.indirect,
                    "total": w.total(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_weeks)?);
    } else {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS);

        table.set_header(vec![
            "Week",
            Category::IndividualSupervision.display_name(),
            Category::GroupSupervision.display_name(),
            Category::Direct.display_name(),
            Category::Indirect.display_name(),
            "Total",
        ]);

        let mut total_ind = 0.0;
        let mut total_grp = 0.0;
        let mut total_direct = 0.0;
        let mut total_indirect = 0.0;

        for w in weeks {
            let week_label = format!(
                "{} – {}",
                w.start.format("%b %d"),
                w.end.format("%b %d, %Y")
            );
            table.add_row(vec![
                week_label,
                format!("{:.1}", w.individual_supervision),
                format!("{:.1}", w.group_supervision),
                format!("{:.1}", w.direct),
                format!("{:.1}", w.indirect),
                format!("{:.1}", w.total()),
            ]);

            total_ind += w.individual_supervision;
            total_grp += w.group_supervision;
            total_direct += w.direct;
            total_indirect += w.indirect;
        }

        let grand_total = total_ind + total_grp + total_direct + total_indirect;
        table.add_row(vec![
            Cell::new("TOTALS").add_attribute(Attribute::Bold),
            Cell::new(format!("{total_ind:.1}")).add_attribute(Attribute::Bold),
            Cell::new(format!("{total_grp:.1}")).add_attribute(Attribute::Bold),
            Cell::new(format!("{total_direct:.1}")).add_attribute(Attribute::Bold),
            Cell::new(format!("{total_indirect:.1}")).add_attribute(Attribute::Bold),
            Cell::new(format!("{grand_total:.1}")).add_attribute(Attribute::Bold),
        ]);

        println!("{table}");
    }

    Ok(())
}
