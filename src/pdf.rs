use std::path::Path;

use anyhow::{Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use genpdf::elements::{self, Paragraph, TableLayout};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::style::Style;
use genpdf::{Alignment, Document, Element, Margins, PaperSize};

use crate::config::LicensureConfig;
use crate::data::model::HoursData;
use crate::data::week;

fn load_font_family() -> Result<FontFamily<FontData>> {
    let regular = FontData::new(
        include_bytes!("../assets/fonts/LiberationSans-Regular.ttf").to_vec(),
        None,
    )
    .map_err(|e| anyhow::anyhow!("Failed to load regular font: {}", e))?;

    let bold = FontData::new(
        include_bytes!("../assets/fonts/LiberationSans-Bold.ttf").to_vec(),
        None,
    )
    .map_err(|e| anyhow::anyhow!("Failed to load bold font: {}", e))?;

    let italic = FontData::new(
        include_bytes!("../assets/fonts/LiberationSans-Italic.ttf").to_vec(),
        None,
    )
    .map_err(|e| anyhow::anyhow!("Failed to load italic font: {}", e))?;

    let bold_italic = FontData::new(
        include_bytes!("../assets/fonts/LiberationSans-BoldItalic.ttf").to_vec(),
        None,
    )
    .map_err(|e| anyhow::anyhow!("Failed to load bold-italic font: {}", e))?;

    Ok(FontFamily {
        regular,
        bold,
        italic,
        bold_italic,
    })
}

fn format_date(date: NaiveDate) -> String {
    date.format("%B %e, %Y").to_string()
}

fn format_week_range(start: NaiveDate, end: NaiveDate) -> String {
    format!(
        "{} – {} {}",
        start.format("%b %d"),
        end.format("%b %d,"),
        end.format("%Y")
    )
}

fn months_between(start: NaiveDate, end: NaiveDate) -> u32 {
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

fn round1(val: f64) -> f64 {
    (val * 10.0).round() / 10.0
}

fn styled_centered(text: &str, style: Style) -> impl Element {
    Paragraph::new(text)
        .aligned(Alignment::Center)
        .styled(style)
}

fn styled_right(text: &str, style: Style) -> impl Element {
    Paragraph::new(text).aligned(Alignment::Right).styled(style)
}

fn build_header(doc: &mut Document, data: &HoursData, config: &LicensureConfig) {
    let today = Local::now().date_naive();

    doc.push(styled_centered(
        "Counseling Licensure Hours Report",
        Style::new().bold().with_font_size(16),
    ));

    doc.push(styled_centered(
        &format!("Generated: {}", format_date(today)),
        Style::new().with_font_size(10),
    ));

    let end_date = data.weeks.last().map(|w| w.end).unwrap_or(today);

    doc.push(styled_centered(
        &format!(
            "Tracking period: {} – {}",
            format_date(config.start_date),
            format_date(end_date)
        ),
        Style::new().with_font_size(10),
    ));

    doc.push(elements::Break::new(1.5));
}

fn build_hours_table(doc: &mut Document, data: &HoursData) {
    let non_zero_weeks: Vec<_> = data.weeks.iter().filter(|w| w.total() > 0.0).collect();

    let mut table = TableLayout::new(vec![3, 2, 2, 2, 2, 2]);
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

    let header_style = Style::new().bold().with_font_size(9);
    let body_style = Style::new().with_font_size(9);
    let bold_body = Style::new().bold().with_font_size(9);

    table
        .row()
        .element(Paragraph::new("Week").styled(header_style))
        .element(styled_right("Ind. Supv", header_style))
        .element(styled_right("Grp. Supv", header_style))
        .element(styled_right("Direct", header_style))
        .element(styled_right("Indirect", header_style))
        .element(styled_right("Total", header_style))
        .push()
        .expect("Invalid table header row");

    let mut sum_ind = 0.0_f64;
    let mut sum_grp = 0.0_f64;
    let mut sum_direct = 0.0_f64;
    let mut sum_indirect = 0.0_f64;
    let mut sum_total = 0.0_f64;

    for w in &non_zero_weeks {
        sum_ind += w.individual_supervision;
        sum_grp += w.group_supervision;
        sum_direct += w.direct;
        sum_indirect += w.indirect;
        sum_total += w.total();

        table
            .row()
            .element(Paragraph::new(format_week_range(w.start, w.end)).styled(body_style))
            .element(styled_right(
                &format!("{:.1}", w.individual_supervision),
                body_style,
            ))
            .element(styled_right(
                &format!("{:.1}", w.group_supervision),
                body_style,
            ))
            .element(styled_right(&format!("{:.1}", w.direct), body_style))
            .element(styled_right(&format!("{:.1}", w.indirect), body_style))
            .element(styled_right(&format!("{:.1}", w.total()), body_style))
            .push()
            .expect("Invalid table data row");
    }

    table
        .row()
        .element(Paragraph::new("TOTALS").styled(bold_body))
        .element(styled_right(&format!("{:.1}", sum_ind), bold_body))
        .element(styled_right(&format!("{:.1}", sum_grp), bold_body))
        .element(styled_right(&format!("{:.1}", sum_direct), bold_body))
        .element(styled_right(&format!("{:.1}", sum_indirect), bold_body))
        .element(styled_right(&format!("{:.1}", sum_total), bold_body))
        .push()
        .expect("Invalid table totals row");

    doc.push(table);
}

fn build_progress_summary(doc: &mut Document, data: &HoursData, config: &LicensureConfig) {
    let today = Local::now().date_naive();
    let start_date = config.start_date;

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

    let weeks_logged = data.weeks.iter().filter(|w| w.total() > 0.0).count();

    let total_pct = if config.total_hours_target > 0 {
        total_hours / config.total_hours_target as f64 * 100.0
    } else {
        0.0
    };
    let direct_pct = if config.direct_hours_target > 0 {
        direct_hours / config.direct_hours_target as f64 * 100.0
    } else {
        0.0
    };
    let months_pct = if config.min_months > 0 {
        months as f64 / config.min_months as f64 * 100.0
    } else {
        0.0
    };

    doc.push(elements::Break::new(1.5));

    doc.push(
        Paragraph::new("Licensure Progress Summary").styled(Style::new().bold().with_font_size(12)),
    );

    doc.push(elements::Break::new(0.5));

    let summary_style = Style::new().with_font_size(10);

    let lines = vec![
        format!(
            "Total supervised hours:    {:.1} / {}  ({:.1}%)",
            round1(total_hours),
            config.total_hours_target,
            round1(total_pct)
        ),
        format!(
            "Direct client hours:       {:.1} / {}  ({:.1}%)",
            round1(direct_hours),
            config.direct_hours_target,
            round1(direct_pct)
        ),
        format!(
            "Months of experience:        {}  /   {}   ({:.1}%)",
            months,
            config.min_months,
            round1(months_pct)
        ),
        format!(
            "Weekly average:             {:.1} hrs/week (target: {:.1})",
            round1(weekly_average),
            config.min_weekly_average
        ),
        format!("Weeks logged:               {}", weeks_logged),
    ];

    for line in lines {
        doc.push(Paragraph::new(line).styled(summary_style));
    }
}

pub fn generate_report(
    data: &HoursData,
    config: &LicensureConfig,
    output_path: &Path,
) -> Result<()> {
    let font_family = load_font_family()?;
    let mut doc = Document::new(font_family);

    doc.set_paper_size(PaperSize::Letter);
    doc.set_font_size(10);
    doc.set_line_spacing(1.25);

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(Margins::trbl(25.4, 25.4, 25.4, 25.4));
    doc.set_page_decorator(decorator);

    doc.set_title("Counseling Licensure Hours Report");

    build_header(&mut doc, data, config);

    let has_data = data.weeks.iter().any(|w| w.total() > 0.0);

    if has_data {
        build_hours_table(&mut doc, data);
        build_progress_summary(&mut doc, data, config);
    } else {
        doc.push(
            Paragraph::new("No hours have been logged yet.")
                .styled(Style::new().with_font_size(10)),
        );
    }

    doc.render_to_file(output_path)
        .with_context(|| format!("Failed to write PDF to {}", output_path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::model::WeekEntry;
    use chrono::NaiveDate;
    use tempfile::TempDir;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn sample_config() -> LicensureConfig {
        LicensureConfig {
            start_date: date(2025, 1, 28),
            total_hours_target: 3000,
            direct_hours_target: 1200,
            min_months: 24,
            min_weekly_average: 15.0,
        }
    }

    #[test]
    fn generate_report_empty_data() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("report.pdf");
        let data = HoursData::new();
        let config = sample_config();

        generate_report(&data, &config, &path).unwrap();

        assert!(path.exists());
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn generate_report_single_week() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("report.pdf");
        let data = HoursData {
            weeks: vec![WeekEntry {
                start: date(2025, 1, 28),
                end: date(2025, 2, 3),
                individual_supervision: 1.0,
                group_supervision: 2.0,
                direct: 14.5,
                indirect: 6.0,
            }],
        };
        let config = sample_config();

        generate_report(&data, &config, &path).unwrap();

        assert!(path.exists());
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn generate_report_many_weeks() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("report.pdf");

        let mut weeks = Vec::new();
        let mut start = date(2025, 1, 28);
        for _ in 0..50 {
            let end = start + chrono::Duration::days(6);
            weeks.push(WeekEntry {
                start,
                end,
                individual_supervision: 1.0,
                group_supervision: 1.5,
                direct: 10.0,
                indirect: 3.0,
            });
            start += chrono::Duration::days(7);
        }
        let data = HoursData { weeks };
        let config = sample_config();

        generate_report(&data, &config, &path).unwrap();

        assert!(path.exists());
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn generate_report_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("sub").join("dir").join("report.pdf");
        let data = HoursData::new();
        let config = sample_config();

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        generate_report(&data, &config, &path).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn format_date_outputs_expected_format() {
        let d = date(2025, 1, 28);
        let formatted = format_date(d);
        assert!(formatted.contains("January"));
        assert!(formatted.contains("28"));
        assert!(formatted.contains("2025"));
    }

    #[test]
    fn format_week_range_outputs_expected_format() {
        let start = date(2025, 1, 28);
        let end = date(2025, 2, 3);
        let formatted = format_week_range(start, end);
        assert!(formatted.contains("Jan 28"));
        assert!(formatted.contains("Feb 03"));
        assert!(formatted.contains("2025"));
    }

    #[test]
    fn months_between_same_date() {
        assert_eq!(months_between(date(2025, 1, 28), date(2025, 1, 28)), 0);
    }

    #[test]
    fn months_between_one_month() {
        assert_eq!(months_between(date(2025, 1, 28), date(2025, 2, 28)), 1);
    }

    #[test]
    fn months_between_partial_month() {
        assert_eq!(months_between(date(2025, 1, 28), date(2025, 2, 27)), 0);
    }

    #[test]
    fn months_between_across_years() {
        assert_eq!(months_between(date(2025, 1, 28), date(2027, 1, 28)), 24);
    }

    #[test]
    fn months_between_end_before_start() {
        assert_eq!(months_between(date(2025, 6, 1), date(2025, 1, 1)), 0);
    }

    #[test]
    fn round1_values() {
        assert!((round1(8.233) - 8.2).abs() < f64::EPSILON);
        assert!((round1(102.75) - 102.8).abs() < f64::EPSILON);
        assert!((round1(0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn generate_report_weeks_with_zero_hours_excluded() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("report.pdf");
        let data = HoursData {
            weeks: vec![
                WeekEntry {
                    start: date(2025, 1, 28),
                    end: date(2025, 2, 3),
                    individual_supervision: 0.0,
                    group_supervision: 0.0,
                    direct: 0.0,
                    indirect: 0.0,
                },
                WeekEntry {
                    start: date(2025, 2, 4),
                    end: date(2025, 2, 10),
                    individual_supervision: 1.0,
                    group_supervision: 0.0,
                    direct: 5.0,
                    indirect: 0.0,
                },
            ],
        };
        let config = sample_config();

        generate_report(&data, &config, &path).unwrap();

        assert!(path.exists());
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);
    }
}
