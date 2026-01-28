use chrono::{Datelike, Duration, NaiveDate, Weekday};

pub fn week_containing(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let weekday_num = date.weekday().num_days_from_monday(); // Mon=0, Tue=1, ..., Sun=6
    let days_since_tuesday = (weekday_num + 6) % 7; // Tue=0, Wed=1, ..., Mon=6
    let start = date - Duration::days(days_since_tuesday as i64);
    let end = start + Duration::days(6);
    (start, end)
}

pub fn current_week(today: NaiveDate) -> (NaiveDate, NaiveDate) {
    week_containing(today)
}

pub fn all_weeks(start_date: NaiveDate, today: NaiveDate) -> Vec<(NaiveDate, NaiveDate)> {
    let (current_start, _) = week_containing(today);
    let mut weeks = Vec::new();
    let mut week_start = start_date;
    while week_start <= current_start {
        let week_end = week_start + Duration::days(6);
        weeks.push((week_start, week_end));
        week_start += Duration::days(7);
    }
    weeks
}

pub fn is_tuesday(date: NaiveDate) -> bool {
    date.weekday() == Weekday::Tue
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_week_containing_tuesday() {
        let (start, end) = week_containing(date(2025, 1, 28));
        assert_eq!(start, date(2025, 1, 28));
        assert_eq!(end, date(2025, 2, 3));
        assert_eq!(start.weekday(), Weekday::Tue);
        assert_eq!(end.weekday(), Weekday::Mon);
    }

    #[test]
    fn test_week_containing_thursday() {
        let (start, end) = week_containing(date(2025, 1, 30));
        assert_eq!(start, date(2025, 1, 28));
        assert_eq!(end, date(2025, 2, 3));
    }

    #[test]
    fn test_week_containing_monday() {
        let (start, end) = week_containing(date(2025, 2, 3));
        assert_eq!(start, date(2025, 1, 28));
        assert_eq!(end, date(2025, 2, 3));
    }

    #[test]
    fn test_week_containing_next_tuesday() {
        let (start, end) = week_containing(date(2025, 2, 4));
        assert_eq!(start, date(2025, 2, 4));
        assert_eq!(end, date(2025, 2, 10));
    }

    #[test]
    fn test_week_containing_wednesday() {
        let (start, end) = week_containing(date(2025, 1, 29));
        assert_eq!(start, date(2025, 1, 28));
        assert_eq!(end, date(2025, 2, 3));
    }

    #[test]
    fn test_week_containing_sunday() {
        let (start, end) = week_containing(date(2025, 2, 2));
        assert_eq!(start, date(2025, 1, 28));
        assert_eq!(end, date(2025, 2, 3));
    }

    #[test]
    fn test_week_containing_saturday() {
        let (start, end) = week_containing(date(2025, 2, 1));
        assert_eq!(start, date(2025, 1, 28));
        assert_eq!(end, date(2025, 2, 3));
    }

    #[test]
    fn test_week_containing_friday() {
        let (start, end) = week_containing(date(2025, 1, 31));
        assert_eq!(start, date(2025, 1, 28));
        assert_eq!(end, date(2025, 2, 3));
    }

    #[test]
    fn test_current_week_is_same_as_week_containing() {
        let today = date(2025, 1, 30);
        assert_eq!(current_week(today), week_containing(today));
    }

    #[test]
    fn test_all_weeks_single_week() {
        let start = date(2025, 1, 28);
        let today = date(2025, 1, 30);
        let weeks = all_weeks(start, today);
        assert_eq!(weeks.len(), 1);
        assert_eq!(weeks[0].0, date(2025, 1, 28));
        assert_eq!(weeks[0].1, date(2025, 2, 3));
    }

    #[test]
    fn test_all_weeks_multiple_weeks() {
        let start = date(2025, 1, 28);
        let today = date(2025, 2, 12); // Wed of 3rd week
        let weeks = all_weeks(start, today);
        assert_eq!(weeks.len(), 3);
        assert_eq!(weeks[0].0, date(2025, 1, 28));
        assert_eq!(weeks[1].0, date(2025, 2, 4));
        assert_eq!(weeks[2].0, date(2025, 2, 11));
    }

    #[test]
    fn test_all_weeks_today_is_start() {
        let start = date(2025, 1, 28);
        let today = date(2025, 1, 28);
        let weeks = all_weeks(start, today);
        assert_eq!(weeks.len(), 1);
    }

    #[test]
    fn test_all_weeks_today_is_monday_end_of_week() {
        let start = date(2025, 1, 28);
        let today = date(2025, 2, 3); // Monday, end of first week
        let weeks = all_weeks(start, today);
        assert_eq!(weeks.len(), 1);
    }

    #[test]
    fn test_all_weeks_today_is_next_tuesday() {
        let start = date(2025, 1, 28);
        let today = date(2025, 2, 4); // Tuesday, start of second week
        let weeks = all_weeks(start, today);
        assert_eq!(weeks.len(), 2);
    }

    #[test]
    fn test_all_weeks_start_always_tuesday() {
        let start = date(2025, 1, 28);
        let today = date(2025, 3, 15);
        let weeks = all_weeks(start, today);
        for (s, e) in &weeks {
            assert_eq!(s.weekday(), Weekday::Tue);
            assert_eq!(e.weekday(), Weekday::Mon);
            assert_eq!(*e - *s, Duration::days(6));
        }
    }

    #[test]
    fn test_all_weeks_consecutive() {
        let start = date(2025, 1, 28);
        let today = date(2025, 3, 15);
        let weeks = all_weeks(start, today);
        for i in 1..weeks.len() {
            assert_eq!(weeks[i].0 - weeks[i - 1].0, Duration::days(7));
        }
    }

    #[test]
    fn test_is_tuesday() {
        assert!(is_tuesday(date(2025, 1, 28)));
        assert!(!is_tuesday(date(2025, 1, 29)));
        assert!(!is_tuesday(date(2025, 1, 27)));
        assert!(is_tuesday(date(2025, 2, 4)));
    }

    #[test]
    fn test_week_containing_all_days_of_week() {
        // Every day from Tue Jan 28 through Mon Feb 3 should map to the same week
        let expected_start = date(2025, 1, 28);
        let expected_end = date(2025, 2, 3);
        for d in 28..=31 {
            let (s, e) = week_containing(date(2025, 1, d));
            assert_eq!(s, expected_start, "Failed for Jan {}", d);
            assert_eq!(e, expected_end, "Failed for Jan {}", d);
        }
        for d in 1..=3 {
            let (s, e) = week_containing(date(2025, 2, d));
            assert_eq!(s, expected_start, "Failed for Feb {}", d);
            assert_eq!(e, expected_end, "Failed for Feb {}", d);
        }
    }
}
