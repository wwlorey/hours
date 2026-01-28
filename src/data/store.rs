use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Context, Result};
use chrono::{Datelike, Weekday};

use super::model::HoursData;

pub fn load(path: &Path) -> Result<HoursData> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let data: HoursData = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(data)
}

pub fn save(path: &Path, data: &HoursData) -> Result<()> {
    let mut data = data.clone();
    validate_and_sort(&mut data)?;

    let json = serde_json::to_string_pretty(&data).context("Failed to serialize data")?;

    let tmp_path = path.with_extension("json.tmp");

    let mut file = File::create(&tmp_path)
        .with_context(|| format!("Failed to create {}", tmp_path.display()))?;
    file.write_all(json.as_bytes())
        .with_context(|| format!("Failed to write {}", tmp_path.display()))?;
    file.write_all(b"\n")?;
    file.sync_all()
        .with_context(|| format!("Failed to fsync {}", tmp_path.display()))?;
    drop(file);

    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "Failed to rename {} to {}",
            tmp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}

fn validate_and_sort(data: &mut HoursData) -> Result<()> {
    for entry in &data.weeks {
        if entry.start.weekday() != Weekday::Tue {
            bail!("Week start {} is not a Tuesday", entry.start);
        }

        let expected_end = entry.start + chrono::Duration::days(6);
        if entry.end != expected_end {
            bail!(
                "Week end {} does not match expected {} (start + 6 days)",
                entry.end,
                expected_end
            );
        }

        if entry.individual_supervision < 0.0
            || entry.group_supervision < 0.0
            || entry.direct < 0.0
            || entry.indirect < 0.0
        {
            bail!("Negative hour values in week starting {}", entry.start);
        }
    }

    data.weeks.sort_by_key(|w| w.start);

    for i in 1..data.weeks.len() {
        if data.weeks[i].start == data.weeks[i - 1].start {
            bail!("Duplicate week starting {}", data.weeks[i].start);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::model::{HoursData, WeekEntry};
    use chrono::NaiveDate;
    use std::fs;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn sample_data() -> HoursData {
        HoursData {
            weeks: vec![
                WeekEntry {
                    start: date(2025, 2, 4),
                    end: date(2025, 2, 10),
                    individual_supervision: 1.0,
                    group_supervision: 0.0,
                    direct: 10.0,
                    indirect: 3.0,
                },
                WeekEntry {
                    start: date(2025, 1, 28),
                    end: date(2025, 2, 3),
                    individual_supervision: 1.0,
                    group_supervision: 2.0,
                    direct: 14.5,
                    indirect: 6.0,
                },
            ],
        }
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hours.json");

        let data = sample_data();
        save(&path, &data).unwrap();

        let loaded = load(&path).unwrap();
        assert_eq!(loaded.weeks.len(), 2);
        // Verify sorted by start date
        assert!(loaded.weeks[0].start < loaded.weeks[1].start);
        assert_eq!(loaded.weeks[0].start, date(2025, 1, 28));
        assert_eq!(loaded.weeks[1].start, date(2025, 2, 4));
    }

    #[test]
    fn test_save_sorts_weeks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hours.json");

        let data = sample_data();
        // weeks are out of order in sample_data
        assert!(data.weeks[0].start > data.weeks[1].start);

        save(&path, &data).unwrap();
        let loaded = load(&path).unwrap();
        assert!(loaded.weeks[0].start < loaded.weeks[1].start);
    }

    #[test]
    fn test_save_validates_tuesday_start() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hours.json");

        let data = HoursData {
            weeks: vec![WeekEntry {
                start: date(2025, 1, 29), // Wednesday
                end: date(2025, 2, 4),
                individual_supervision: 0.0,
                group_supervision: 0.0,
                direct: 0.0,
                indirect: 0.0,
            }],
        };
        assert!(save(&path, &data).is_err());
    }

    #[test]
    fn test_save_validates_end_date() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hours.json");

        let data = HoursData {
            weeks: vec![WeekEntry {
                start: date(2025, 1, 28),
                end: date(2025, 2, 4), // Wrong: should be Feb 3
                individual_supervision: 0.0,
                group_supervision: 0.0,
                direct: 0.0,
                indirect: 0.0,
            }],
        };
        assert!(save(&path, &data).is_err());
    }

    #[test]
    fn test_save_validates_negative_hours() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hours.json");

        let data = HoursData {
            weeks: vec![WeekEntry {
                start: date(2025, 1, 28),
                end: date(2025, 2, 3),
                individual_supervision: -1.0,
                group_supervision: 0.0,
                direct: 0.0,
                indirect: 0.0,
            }],
        };
        assert!(save(&path, &data).is_err());
    }

    #[test]
    fn test_save_validates_duplicate_weeks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hours.json");

        let data = HoursData {
            weeks: vec![
                WeekEntry::new(date(2025, 1, 28), date(2025, 2, 3)),
                WeekEntry::new(date(2025, 1, 28), date(2025, 2, 3)),
            ],
        };
        assert!(save(&path, &data).is_err());
    }

    #[test]
    fn test_save_empty_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hours.json");

        let data = HoursData::new();
        save(&path, &data).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"weeks\": []"));

        let loaded = load(&path).unwrap();
        assert!(loaded.weeks.is_empty());
    }

    #[test]
    fn test_save_atomic_no_tmp_left() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hours.json");

        let data = HoursData::new();
        save(&path, &data).unwrap();

        let tmp_path = path.with_extension("json.tmp");
        assert!(!tmp_path.exists());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        assert!(load(&path).is_err());
    }

    #[test]
    fn test_load_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hours.json");
        fs::write(&path, "not valid json").unwrap();
        assert!(load(&path).is_err());
    }

    #[test]
    fn test_save_preserves_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hours.json");

        let data = HoursData {
            weeks: vec![WeekEntry {
                start: date(2025, 1, 28),
                end: date(2025, 2, 3),
                individual_supervision: 1.5,
                group_supervision: 2.25,
                direct: 14.75,
                indirect: 6.0,
            }],
        };
        save(&path, &data).unwrap();
        let loaded = load(&path).unwrap();

        let w = &loaded.weeks[0];
        assert!((w.individual_supervision - 1.5).abs() < f64::EPSILON);
        assert!((w.group_supervision - 2.25).abs() < f64::EPSILON);
        assert!((w.direct - 14.75).abs() < f64::EPSILON);
        assert!((w.indirect - 6.0).abs() < f64::EPSILON);
    }
}
