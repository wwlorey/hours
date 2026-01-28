use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoursData {
    pub weeks: Vec<WeekEntry>,
}

impl HoursData {
    pub fn new() -> Self {
        Self { weeks: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekEntry {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub individual_supervision: f64,
    pub group_supervision: f64,
    pub direct: f64,
    pub indirect: f64,
}

impl WeekEntry {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            start,
            end,
            individual_supervision: 0.0,
            group_supervision: 0.0,
            direct: 0.0,
            indirect: 0.0,
        }
    }

    pub fn total(&self) -> f64 {
        self.individual_supervision + self.group_supervision + self.direct + self.indirect
    }

    pub fn get(&self, category: Category) -> f64 {
        match category {
            Category::IndividualSupervision => self.individual_supervision,
            Category::GroupSupervision => self.group_supervision,
            Category::Direct => self.direct,
            Category::Indirect => self.indirect,
        }
    }

    pub fn set(&mut self, category: Category, value: f64) {
        match category {
            Category::IndividualSupervision => self.individual_supervision = value,
            Category::GroupSupervision => self.group_supervision = value,
            Category::Direct => self.direct = value,
            Category::Indirect => self.indirect = value,
        }
    }

    pub fn add(&mut self, category: Category, value: f64) {
        match category {
            Category::IndividualSupervision => self.individual_supervision += value,
            Category::GroupSupervision => self.group_supervision += value,
            Category::Direct => self.direct += value,
            Category::Indirect => self.indirect += value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    IndividualSupervision,
    GroupSupervision,
    Direct,
    Indirect,
}

impl Category {
    pub const ALL: [Category; 4] = [
        Category::IndividualSupervision,
        Category::GroupSupervision,
        Category::Direct,
        Category::Indirect,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            Category::IndividualSupervision => "Ind Sv",
            Category::GroupSupervision => "Grp Sv",
            Category::Direct => "Direct",
            Category::Indirect => "Indirect",
        }
    }

    pub fn long_name(&self) -> &'static str {
        match self {
            Category::IndividualSupervision => "Individual Supervision",
            Category::GroupSupervision => "Group Supervision",
            Category::Direct => "Direct (client contact)",
            Category::Indirect => "Indirect",
        }
    }
}

impl FromStr for Category {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "individual_supervision" => Ok(Category::IndividualSupervision),
            "group_supervision" => Ok(Category::GroupSupervision),
            "direct" => Ok(Category::Direct),
            "indirect" => Ok(Category::Indirect),
            _ => Err(anyhow::anyhow!(
                "Invalid category '{}'. Valid categories: individual_supervision, group_supervision, direct, indirect",
                s
            )),
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::IndividualSupervision => write!(f, "individual_supervision"),
            Category::GroupSupervision => write!(f, "group_supervision"),
            Category::Direct => write!(f, "direct"),
            Category::Indirect => write!(f, "indirect"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_week_entry_total() {
        let entry = WeekEntry {
            start: NaiveDate::from_ymd_opt(2025, 1, 28).unwrap(),
            end: NaiveDate::from_ymd_opt(2025, 2, 3).unwrap(),
            individual_supervision: 1.0,
            group_supervision: 2.0,
            direct: 14.5,
            indirect: 6.0,
        };
        assert!((entry.total() - 23.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_week_entry_new_zeros() {
        let start = NaiveDate::from_ymd_opt(2025, 1, 28).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 2, 3).unwrap();
        let entry = WeekEntry::new(start, end);
        assert!((entry.total() - 0.0).abs() < f64::EPSILON);
        assert_eq!(entry.start, start);
        assert_eq!(entry.end, end);
    }

    #[test]
    fn test_week_entry_get_set() {
        let start = NaiveDate::from_ymd_opt(2025, 1, 28).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 2, 3).unwrap();
        let mut entry = WeekEntry::new(start, end);

        for cat in Category::ALL {
            assert!((entry.get(cat) - 0.0).abs() < f64::EPSILON);
        }

        entry.set(Category::Direct, 5.0);
        assert!((entry.get(Category::Direct) - 5.0).abs() < f64::EPSILON);

        entry.add(Category::Direct, 2.5);
        assert!((entry.get(Category::Direct) - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_category_from_str() {
        assert_eq!(
            "individual_supervision".parse::<Category>().unwrap(),
            Category::IndividualSupervision
        );
        assert_eq!(
            "group_supervision".parse::<Category>().unwrap(),
            Category::GroupSupervision
        );
        assert_eq!("direct".parse::<Category>().unwrap(), Category::Direct);
        assert_eq!("indirect".parse::<Category>().unwrap(), Category::Indirect);
        assert!("invalid".parse::<Category>().is_err());
    }

    #[test]
    fn test_category_display_roundtrip() {
        for cat in Category::ALL {
            let s = cat.to_string();
            let parsed: Category = s.parse().unwrap();
            assert_eq!(parsed, cat);
        }
    }

    #[test]
    fn test_hours_data_new_empty() {
        let data = HoursData::new();
        assert!(data.weeks.is_empty());
    }

    #[test]
    fn test_hours_data_serde_roundtrip() {
        let data = HoursData {
            weeks: vec![WeekEntry {
                start: NaiveDate::from_ymd_opt(2025, 1, 28).unwrap(),
                end: NaiveDate::from_ymd_opt(2025, 2, 3).unwrap(),
                individual_supervision: 1.0,
                group_supervision: 2.0,
                direct: 14.5,
                indirect: 6.0,
            }],
        };
        let json = serde_json::to_string_pretty(&data).unwrap();
        let deserialized: HoursData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.weeks.len(), 1);
        assert!((deserialized.weeks[0].total() - 23.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hours_data_empty_serde() {
        let data = HoursData::new();
        let json = serde_json::to_string_pretty(&data).unwrap();
        assert!(json.contains("\"weeks\": []"));
        let deserialized: HoursData = serde_json::from_str(&json).unwrap();
        assert!(deserialized.weeks.is_empty());
    }
}
