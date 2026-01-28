use std::io::{self, Write};

use anyhow::{bail, Result};
use chrono::NaiveDate;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{self, Stylize},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};

use crate::data::model::{Category, HoursData};
use crate::data::week;

struct RawModeGuard;

impl RawModeGuard {
    fn enable() -> Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

enum SelectAction {
    Up,
    Down,
    Top,
    Bottom,
    Confirm,
    Cancel,
}

fn read_select_key() -> Result<SelectAction> {
    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                return Ok(SelectAction::Cancel);
            }
            match code {
                KeyCode::Char('j') | KeyCode::Down => return Ok(SelectAction::Down),
                KeyCode::Char('k') | KeyCode::Up => return Ok(SelectAction::Up),
                KeyCode::Char('g') => return Ok(SelectAction::Top),
                KeyCode::Char('G') => return Ok(SelectAction::Bottom),
                KeyCode::Enter => return Ok(SelectAction::Confirm),
                KeyCode::Esc | KeyCode::Char('q') => return Ok(SelectAction::Cancel),
                _ => {}
            }
        }
    }
}

fn render_list(
    stdout: &mut io::Stdout,
    header: &str,
    items: &[String],
    selected: usize,
) -> Result<()> {
    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.queue(terminal::Clear(ClearType::All))?;

    stdout.queue(style::PrintStyledContent(header.bold()))?;
    stdout.queue(cursor::MoveToNextLine(1))?;

    for (i, item) in items.iter().enumerate() {
        if i == selected {
            stdout.queue(style::PrintStyledContent("  > ".green()))?;
            stdout.queue(style::PrintStyledContent(item.as_str().green()))?;
        } else {
            stdout.queue(style::Print(format!("    {item}")))?;
        }
        stdout.queue(cursor::MoveToNextLine(1))?;
    }

    stdout.flush()?;
    Ok(())
}

fn select_from_list(header: &str, items: &[String], initial: usize) -> Result<Option<usize>> {
    if items.is_empty() {
        bail!("No items to select from");
    }

    let _guard = RawModeGuard::enable()?;
    let mut stdout = io::stdout();
    stdout.execute(cursor::Hide)?;

    let mut selected = initial.min(items.len() - 1);
    render_list(&mut stdout, header, items, selected)?;

    let result = loop {
        match read_select_key()? {
            SelectAction::Down => {
                if selected < items.len() - 1 {
                    selected += 1;
                    render_list(&mut stdout, header, items, selected)?;
                }
            }
            SelectAction::Up => {
                if selected > 0 {
                    selected -= 1;
                    render_list(&mut stdout, header, items, selected)?;
                }
            }
            SelectAction::Top => {
                selected = 0;
                render_list(&mut stdout, header, items, selected)?;
            }
            SelectAction::Bottom => {
                selected = items.len() - 1;
                render_list(&mut stdout, header, items, selected)?;
            }
            SelectAction::Confirm => break Some(selected),
            SelectAction::Cancel => break None,
        }
    };

    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::Clear(ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;

    Ok(result)
}

fn format_week_label(
    start: NaiveDate,
    end: NaiveDate,
    is_current: bool,
    data: &HoursData,
) -> String {
    let total = data
        .weeks
        .iter()
        .find(|w| w.start == start)
        .map(|w| w.total())
        .unwrap_or(0.0);

    let date_range = format!("{} – {}", start.format("%b %d"), end.format("%b %d, %Y"));

    let current_marker = if is_current { " (current)" } else { "" };
    format!("{date_range}{current_marker}    {total:.1} hrs")
}

pub fn select_week(
    weeks: &[(NaiveDate, NaiveDate)],
    data: &HoursData,
    current_week_start: NaiveDate,
) -> Result<Option<NaiveDate>> {
    let items: Vec<String> = weeks
        .iter()
        .rev()
        .map(|(start, end)| format_week_label(*start, *end, *start == current_week_start, data))
        .collect();

    let current_index = weeks
        .iter()
        .rev()
        .position(|(start, _)| *start == current_week_start)
        .unwrap_or(0);

    match select_from_list("Select week:", &items, current_index)? {
        Some(idx) => {
            let reversed_idx = weeks.len() - 1 - idx;
            Ok(Some(weeks[reversed_idx].0))
        }
        None => Ok(None),
    }
}

pub fn select_category() -> Result<Option<Category>> {
    let items: Vec<String> = Category::ALL
        .iter()
        .map(|c| c.long_name().to_string())
        .collect();

    match select_from_list("Select category:", &items, 0)? {
        Some(idx) => Ok(Some(Category::ALL[idx])),
        None => Ok(None),
    }
}

pub fn input_hours(prompt: &str, current_value: Option<f64>) -> Result<Option<f64>> {
    let _guard = RawModeGuard::enable()?;
    let mut stdout = io::stdout();

    let display_prompt = match current_value {
        Some(val) => format!("{prompt} [{val:.1}]: "),
        None => format!("{prompt}: "),
    };

    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.queue(style::Print(&display_prompt))?;
    stdout.flush()?;

    let mut input = String::new();

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                stdout.execute(cursor::MoveToNextLine(1))?;
                return Ok(None);
            }
            match code {
                KeyCode::Enter => {
                    stdout.execute(cursor::MoveToNextLine(1))?;
                    if input.is_empty() {
                        return Ok(current_value);
                    }
                    match input.parse::<f64>() {
                        Ok(val) if val >= 0.0 => return Ok(Some(val)),
                        Ok(_) => {
                            stdout.queue(style::PrintStyledContent(
                                "Hours must be >= 0. Try again.".red(),
                            ))?;
                            stdout.queue(cursor::MoveToNextLine(1))?;
                            input.clear();
                            stdout.queue(style::Print(&display_prompt))?;
                            stdout.flush()?;
                        }
                        Err(_) => {
                            stdout.queue(style::PrintStyledContent(
                                "Invalid number. Try again.".red(),
                            ))?;
                            stdout.queue(cursor::MoveToNextLine(1))?;
                            input.clear();
                            stdout.queue(style::Print(&display_prompt))?;
                            stdout.flush()?;
                        }
                    }
                }
                KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                    input.push(c);
                    stdout.queue(style::Print(c))?;
                    stdout.flush()?;
                }
                KeyCode::Backspace => {
                    if !input.is_empty() {
                        input.pop();
                        stdout.queue(cursor::MoveLeft(1))?;
                        stdout.queue(style::Print(' '))?;
                        stdout.queue(cursor::MoveLeft(1))?;
                        stdout.flush()?;
                    }
                }
                KeyCode::Esc => {
                    stdout.execute(cursor::MoveToNextLine(1))?;
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

pub fn input_text(prompt: &str, default: Option<&str>) -> Result<Option<String>> {
    let _guard = RawModeGuard::enable()?;
    let mut stdout = io::stdout();

    let display_prompt = match default {
        Some(d) => format!("{prompt} [{d}]: "),
        None => format!("{prompt}: "),
    };

    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.queue(style::Print(&display_prompt))?;
    stdout.flush()?;

    let mut input = String::new();

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                stdout.execute(cursor::MoveToNextLine(1))?;
                return Ok(None);
            }
            match code {
                KeyCode::Enter => {
                    stdout.execute(cursor::MoveToNextLine(1))?;
                    if input.is_empty() {
                        return Ok(default.map(|s| s.to_string()));
                    }
                    return Ok(Some(input));
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    stdout.queue(style::Print(c))?;
                    stdout.flush()?;
                }
                KeyCode::Backspace => {
                    if !input.is_empty() {
                        input.pop();
                        stdout.queue(cursor::MoveLeft(1))?;
                        stdout.queue(style::Print(' '))?;
                        stdout.queue(cursor::MoveLeft(1))?;
                        stdout.flush()?;
                    }
                }
                KeyCode::Esc => {
                    stdout.execute(cursor::MoveToNextLine(1))?;
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

pub fn input_date(prompt: &str, must_be_tuesday: bool) -> Result<Option<NaiveDate>> {
    let _guard = RawModeGuard::enable()?;
    let mut stdout = io::stdout();

    let display_prompt = format!("{prompt} (YYYY-MM-DD): ");

    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.queue(style::Print(&display_prompt))?;
    stdout.flush()?;

    let mut input = String::new();

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                stdout.execute(cursor::MoveToNextLine(1))?;
                return Ok(None);
            }
            match code {
                KeyCode::Enter => {
                    stdout.execute(cursor::MoveToNextLine(1))?;
                    if input.is_empty() {
                        stdout.queue(style::PrintStyledContent("Date is required.".red()))?;
                        stdout.queue(cursor::MoveToNextLine(1))?;
                        stdout.queue(style::Print(&display_prompt))?;
                        stdout.flush()?;
                        continue;
                    }
                    match NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
                        Ok(date) => {
                            if must_be_tuesday && !week::is_tuesday(date) {
                                stdout.queue(style::PrintStyledContent(
                                    "Date must be a Tuesday. Try again.".red(),
                                ))?;
                                stdout.queue(cursor::MoveToNextLine(1))?;
                                input.clear();
                                stdout.queue(style::Print(&display_prompt))?;
                                stdout.flush()?;
                            } else {
                                return Ok(Some(date));
                            }
                        }
                        Err(_) => {
                            stdout.queue(style::PrintStyledContent(
                                "Invalid date format. Use YYYY-MM-DD.".red(),
                            ))?;
                            stdout.queue(cursor::MoveToNextLine(1))?;
                            input.clear();
                            stdout.queue(style::Print(&display_prompt))?;
                            stdout.flush()?;
                        }
                    }
                }
                KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => {
                    input.push(c);
                    stdout.queue(style::Print(c))?;
                    stdout.flush()?;
                }
                KeyCode::Backspace => {
                    if !input.is_empty() {
                        input.pop();
                        stdout.queue(cursor::MoveLeft(1))?;
                        stdout.queue(style::Print(' '))?;
                        stdout.queue(cursor::MoveLeft(1))?;
                        stdout.flush()?;
                    }
                }
                KeyCode::Esc => {
                    stdout.execute(cursor::MoveToNextLine(1))?;
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

pub fn confirm(prompt: &str) -> Result<bool> {
    let _guard = RawModeGuard::enable()?;
    let mut stdout = io::stdout();

    stdout.queue(style::Print(format!("{prompt} [Y/n]: ")))?;
    stdout.flush()?;

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                stdout.execute(cursor::MoveToNextLine(1))?;
                return Ok(false);
            }
            match code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    stdout.execute(cursor::MoveToNextLine(1))?;
                    return Ok(true);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    stdout.execute(cursor::MoveToNextLine(1))?;
                    return Ok(false);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_format_week_label_current_with_hours() {
        let data = HoursData {
            weeks: vec![crate::data::model::WeekEntry {
                start: date(2025, 1, 28),
                end: date(2025, 2, 3),
                individual_supervision: 1.0,
                group_supervision: 2.0,
                direct: 14.5,
                indirect: 6.0,
            }],
        };

        let label = format_week_label(date(2025, 1, 28), date(2025, 2, 3), true, &data);
        assert!(label.contains("Jan 28"));
        assert!(label.contains("Feb 03, 2025"));
        assert!(label.contains("(current)"));
        assert!(label.contains("23.5 hrs"));
    }

    #[test]
    fn test_format_week_label_not_current_no_hours() {
        let data = HoursData::new();
        let label = format_week_label(date(2025, 1, 21), date(2025, 1, 27), false, &data);
        assert!(label.contains("Jan 21"));
        assert!(label.contains("Jan 27, 2025"));
        assert!(!label.contains("(current)"));
        assert!(label.contains("0.0 hrs"));
    }

    #[test]
    fn test_format_week_label_not_current_with_hours() {
        let data = HoursData {
            weeks: vec![crate::data::model::WeekEntry {
                start: date(2025, 2, 4),
                end: date(2025, 2, 10),
                individual_supervision: 0.0,
                group_supervision: 0.0,
                direct: 5.0,
                indirect: 3.0,
            }],
        };

        let label = format_week_label(date(2025, 2, 4), date(2025, 2, 10), false, &data);
        assert!(!label.contains("(current)"));
        assert!(label.contains("8.0 hrs"));
    }

    #[test]
    fn test_raw_mode_guard_cleanup() {
        {
            let _guard = RawModeGuard::enable();
        }
        // After the guard is dropped, raw mode should be disabled.
        // We can't easily assert this in a unit test, but we verify
        // the guard can be created and dropped without panicking.
    }

    #[test]
    fn test_category_items_match_all_categories() {
        let items: Vec<String> = Category::ALL
            .iter()
            .map(|c| c.long_name().to_string())
            .collect();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], "Individual Supervision");
        assert_eq!(items[1], "Group Supervision");
        assert_eq!(items[2], "Direct (client contact)");
        assert_eq!(items[3], "Indirect");
    }
}
