use assert_cmd::Command;
use assert_fs::TempDir;
use chrono::Datelike;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;

fn hours_cmd() -> Command {
    Command::cargo_bin("hours").unwrap()
}

fn init_env(config_dir: &TempDir, data_dir: &TempDir) {
    let data_path = data_dir.path().to_str().unwrap();
    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args([
            "--no-git",
            "init",
            "--data-dir",
            data_path,
            "--remote",
            "git@github.com:test/test.git",
            "--start-date",
            "2025-01-28",
            "--non-interactive",
        ])
        .assert()
        .success();
}

fn add_hours(config_dir: &TempDir, data_dir: &TempDir, category: &str, hours: &str) {
    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args([
            "--no-git",
            "add",
            "--category",
            category,
            "--hours",
            hours,
            "--non-interactive",
        ])
        .assert()
        .success();
}

fn add_hours_to_week(
    config_dir: &TempDir,
    data_dir: &TempDir,
    week: &str,
    category: &str,
    hours: &str,
) {
    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args([
            "--no-git",
            "add",
            "--week",
            week,
            "--category",
            category,
            "--hours",
            hours,
            "--non-interactive",
        ])
        .assert()
        .success();
}

fn load_data(data_dir: &TempDir) -> Value {
    let path = data_dir.path().join("hours.json");
    let contents = fs::read_to_string(path).unwrap();
    serde_json::from_str(&contents).unwrap()
}

#[test]
fn initialize_fresh_setup() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();

    let data_path = data_dir.path().to_str().unwrap();
    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args([
            "--no-git",
            "init",
            "--data-dir",
            data_path,
            "--remote",
            "git@github.com:test/test.git",
            "--start-date",
            "2025-01-28",
            "--non-interactive",
        ])
        .assert()
        .success();

    let config_path = config_dir.path().join("config.toml");
    assert!(config_path.exists());
    let config_contents = fs::read_to_string(&config_path).unwrap();
    assert!(config_contents.contains(data_path));
    assert!(config_contents.contains("2025-01-28"));

    let data_file = data_dir.path().join("hours.json");
    assert!(data_file.exists());
    let data: Value = serde_json::from_str(&fs::read_to_string(&data_file).unwrap()).unwrap();
    let weeks = data["weeks"].as_array().unwrap();
    assert!(weeks.is_empty());
}

#[test]
fn add_hours_to_current_week() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours(&config_dir, &data_dir, "direct", "3.5");

    let data = load_data(&data_dir);
    let weeks = data["weeks"].as_array().unwrap();
    assert_eq!(weeks.len(), 1);

    let week = &weeks[0];
    assert_eq!(week["direct"].as_f64().unwrap(), 3.5);
    assert_eq!(week["individual_supervision"].as_f64().unwrap(), 0.0);
    assert_eq!(week["group_supervision"].as_f64().unwrap(), 0.0);
    assert_eq!(week["indirect"].as_f64().unwrap(), 0.0);

    let start = week["start"].as_str().unwrap();
    let start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap();
    assert_eq!(
        start_date.weekday(),
        chrono::Weekday::Tue,
        "Week start must be a Tuesday"
    );

    let end = week["end"].as_str().unwrap();
    let end_date = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d").unwrap();
    assert_eq!(
        end_date.weekday(),
        chrono::Weekday::Mon,
        "Week end must be a Monday"
    );
    assert_eq!((end_date - start_date).num_days(), 6);
}

#[test]
fn add_hours_incrementally() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours(&config_dir, &data_dir, "direct", "3.5");
    add_hours(&config_dir, &data_dir, "direct", "2.0");

    let data = load_data(&data_dir);
    let weeks = data["weeks"].as_array().unwrap();
    assert_eq!(weeks.len(), 1);
    assert!((weeks[0]["direct"].as_f64().unwrap() - 5.5).abs() < f64::EPSILON);
}

#[test]
fn add_hours_multiple_categories() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours(&config_dir, &data_dir, "direct", "3.5");
    add_hours(&config_dir, &data_dir, "individual_supervision", "1.0");
    add_hours(&config_dir, &data_dir, "group_supervision", "2.0");
    add_hours(&config_dir, &data_dir, "indirect", "4.0");

    let data = load_data(&data_dir);
    let weeks = data["weeks"].as_array().unwrap();
    assert_eq!(weeks.len(), 1);

    let w = &weeks[0];
    assert_eq!(w["direct"].as_f64().unwrap(), 3.5);
    assert_eq!(w["individual_supervision"].as_f64().unwrap(), 1.0);
    assert_eq!(w["group_supervision"].as_f64().unwrap(), 2.0);
    assert_eq!(w["indirect"].as_f64().unwrap(), 4.0);

    let total = w["direct"].as_f64().unwrap()
        + w["individual_supervision"].as_f64().unwrap()
        + w["group_supervision"].as_f64().unwrap()
        + w["indirect"].as_f64().unwrap();
    assert!((total - 10.5).abs() < f64::EPSILON);
}

#[test]
fn add_hours_to_specific_past_week() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "direct", "5.0");

    let data = load_data(&data_dir);
    let weeks = data["weeks"].as_array().unwrap();
    assert_eq!(weeks.len(), 1);
    assert_eq!(weeks[0]["start"].as_str().unwrap(), "2025-01-28");
    assert_eq!(weeks[0]["end"].as_str().unwrap(), "2025-02-03");
    assert_eq!(weeks[0]["direct"].as_f64().unwrap(), 5.0);
}

#[test]
fn edit_overwrites_values() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "direct", "3.5");

    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args([
            "--no-git",
            "edit",
            "--week",
            "2025-01-28",
            "--direct",
            "10.0",
            "--non-interactive",
        ])
        .assert()
        .success();

    let data = load_data(&data_dir);
    let weeks = data["weeks"].as_array().unwrap();
    assert_eq!(weeks[0]["direct"].as_f64().unwrap(), 10.0);
}

#[test]
fn edit_preserves_unspecified_categories() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "direct", "3.5");
    add_hours_to_week(
        &config_dir,
        &data_dir,
        "2025-01-28",
        "individual_supervision",
        "1.0",
    );

    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args([
            "--no-git",
            "edit",
            "--week",
            "2025-01-28",
            "--direct",
            "10.0",
            "--non-interactive",
        ])
        .assert()
        .success();

    let data = load_data(&data_dir);
    let weeks = data["weeks"].as_array().unwrap();
    assert_eq!(weeks[0]["direct"].as_f64().unwrap(), 10.0);
    assert_eq!(
        weeks[0]["individual_supervision"].as_f64().unwrap(),
        1.0,
        "Unspecified categories must be preserved"
    );
}

#[test]
fn list_output_table() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "direct", "5.0");
    add_hours_to_week(&config_dir, &data_dir, "2025-02-04", "direct", "3.0");

    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Jan 28"))
        .stdout(predicate::str::contains("Feb 04"))
        .stdout(predicate::str::contains("TOTALS"));
}

#[test]
fn list_output_json() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "direct", "5.0");

    let output = hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["start"].as_str().unwrap(), "2025-01-28");
    assert_eq!(arr[0]["direct"].as_f64().unwrap(), 5.0);
    assert!(arr[0]["total"].as_f64().unwrap() > 0.0);
}

#[test]
fn list_with_last_n() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "direct", "1.0");
    add_hours_to_week(&config_dir, &data_dir, "2025-02-04", "direct", "2.0");
    add_hours_to_week(&config_dir, &data_dir, "2025-02-11", "direct", "3.0");

    let output = hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args(["list", "--last", "2", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["start"].as_str().unwrap(), "2025-02-04");
    assert_eq!(arr[1]["start"].as_str().unwrap(), "2025-02-11");
}

#[test]
fn summary_calculations() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "direct", "10.0");
    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "indirect", "5.0");
    add_hours_to_week(&config_dir, &data_dir, "2025-02-04", "direct", "8.0");

    let output = hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args(["summary", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();

    let total_current = json["total_hours"]["current"].as_f64().unwrap();
    assert!(
        (total_current - 23.0).abs() < 0.1,
        "total_hours should be 23.0, got {total_current}"
    );

    let direct_current = json["direct_hours"]["current"].as_f64().unwrap();
    assert!(
        (direct_current - 18.0).abs() < 0.1,
        "direct_hours should be 18.0, got {direct_current}"
    );

    assert_eq!(json["total_hours"]["target"].as_u64().unwrap(), 3000);
    assert_eq!(json["direct_hours"]["target"].as_u64().unwrap(), 1200);

    let total_pct = json["total_hours"]["percentage"].as_f64().unwrap();
    assert!(total_pct > 0.0);

    assert_eq!(json["start_date"].as_str().unwrap(), "2025-01-28");
}

#[test]
fn summary_empty_state() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    let output = hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args(["summary", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["total_hours"]["current"].as_f64().unwrap(), 0.0);
    assert_eq!(json["direct_hours"]["current"].as_f64().unwrap(), 0.0);
    assert_eq!(json["total_hours"]["percentage"].as_f64().unwrap(), 0.0);
    assert_eq!(json["direct_hours"]["percentage"].as_f64().unwrap(), 0.0);
    assert_eq!(json["weeks_logged"].as_u64().unwrap(), 0);
}

#[test]
fn export_generates_pdf() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "direct", "5.0");

    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args(["--no-git", "export"])
        .assert()
        .success();

    let exports_dir = data_dir.path().join("exports");
    assert!(exports_dir.exists());

    let pdf_files: Vec<_> = fs::read_dir(&exports_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "pdf"))
        .collect();

    assert_eq!(pdf_files.len(), 1, "Expected exactly one PDF file");
    assert!(
        pdf_files[0].metadata().unwrap().len() > 0,
        "PDF file should not be empty"
    );
}

#[test]
fn export_custom_output_path() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "direct", "5.0");

    let custom_path = output_dir.path().join("custom-report.pdf");
    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args([
            "--no-git",
            "export",
            "--output",
            custom_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(custom_path.exists());
    assert!(
        custom_path.metadata().unwrap().len() > 0,
        "PDF file should not be empty"
    );
}

#[test]
fn config_env_var_overrides() {
    let config_dir = TempDir::new().unwrap();
    let data_dir_a = TempDir::new().unwrap();
    let data_dir_b = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir_a);

    // Create a separate hours.json in data_dir_b with known data
    let data_json = r#"{"weeks":[{"start":"2025-01-28","end":"2025-02-03","individual_supervision":0.0,"group_supervision":0.0,"direct":99.0,"indirect":0.0}]}"#;
    fs::write(data_dir_b.path().join("hours.json"), data_json).unwrap();

    let output = hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir_b.path())
        .env("HOURS_NO_GIT", "1")
        .args(["summary", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    let direct = json["direct_hours"]["current"].as_f64().unwrap();
    assert!(
        (direct - 99.0).abs() < 0.1,
        "Should read from HOURS_DATA_DIR override, got {direct}"
    );
}

#[test]
fn validation_rejects_negative_hours() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args([
            "--no-git",
            "add",
            "--category",
            "direct",
            "--hours",
            "-1.0",
            "--non-interactive",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Hours must be >= 0"));
}

#[test]
fn validation_rejects_non_tuesday_week_start() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    // 2025-01-29 is a Wednesday
    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args([
            "--no-git",
            "add",
            "--week",
            "2025-01-29",
            "--category",
            "direct",
            "--hours",
            "1.0",
            "--non-interactive",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Tuesday"));
}

#[test]
fn list_and_summary_empty_state() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No hours logged yet"));

    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args(["summary"])
        .assert()
        .success();
}

#[test]
fn data_file_integrity_after_multiple_operations() {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    init_env(&config_dir, &data_dir);

    add_hours_to_week(&config_dir, &data_dir, "2025-02-11", "direct", "3.0");
    add_hours_to_week(&config_dir, &data_dir, "2025-01-28", "direct", "5.0");
    add_hours_to_week(&config_dir, &data_dir, "2025-02-04", "indirect", "2.0");

    // Edit one of the weeks
    hours_cmd()
        .env("HOURS_CONFIG_DIR", config_dir.path())
        .env("HOURS_DATA_DIR", data_dir.path())
        .env("HOURS_NO_GIT", "1")
        .args([
            "--no-git",
            "edit",
            "--week",
            "2025-01-28",
            "--direct",
            "7.0",
            "--non-interactive",
        ])
        .assert()
        .success();

    // Add to another week again (accumulate)
    add_hours_to_week(&config_dir, &data_dir, "2025-02-04", "direct", "1.0");

    let data = load_data(&data_dir);
    let weeks = data["weeks"].as_array().unwrap();

    // Weeks sorted by start date ascending
    assert_eq!(weeks.len(), 3);
    let starts: Vec<&str> = weeks.iter().map(|w| w["start"].as_str().unwrap()).collect();
    assert_eq!(starts, vec!["2025-01-28", "2025-02-04", "2025-02-11"]);

    // All start dates are Tuesdays, all end dates are start + 6 days
    for w in weeks {
        let start =
            chrono::NaiveDate::parse_from_str(w["start"].as_str().unwrap(), "%Y-%m-%d").unwrap();
        let end =
            chrono::NaiveDate::parse_from_str(w["end"].as_str().unwrap(), "%Y-%m-%d").unwrap();
        assert_eq!(start.weekday(), chrono::Weekday::Tue);
        assert_eq!(end.weekday(), chrono::Weekday::Mon);
        assert_eq!((end - start).num_days(), 6);
    }

    // No duplicate weeks (already guaranteed by having exactly 3 distinct start dates above)

    // Verify edited value
    assert_eq!(weeks[0]["direct"].as_f64().unwrap(), 7.0);

    // Verify accumulated value
    assert_eq!(weeks[1]["indirect"].as_f64().unwrap(), 2.0);
    assert_eq!(weeks[1]["direct"].as_f64().unwrap(), 1.0);
}
