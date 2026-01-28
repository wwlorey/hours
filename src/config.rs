use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data: DataConfig,
    pub git: GitConfig,
    pub licensure: LicensureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub remote: String,
    pub auto_push: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensureConfig {
    pub start_date: NaiveDate,
    pub total_hours_target: u32,
    pub direct_hours_target: u32,
    pub min_months: u32,
    pub min_weekly_average: f64,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        if let Ok(dir) = env::var("HOURS_CONFIG_DIR") {
            PathBuf::from(dir)
        } else {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("~/.config"))
                .join("hours")
        }
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            anyhow::bail!("Configuration not found. Run `hours init` to set up.");
        }
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let mut config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse {}", path.display()))?;

        if let Ok(data_dir) = env::var("HOURS_DATA_DIR") {
            config.data.directory = data_dir;
        }

        if env::var("HOURS_NO_GIT").ok().as_deref() == Some("1") {
            config.git.auto_push = false;
        }

        config.data.directory = expand_tilde(&config.data.directory);

        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory {}", parent.display())
            })?;
        }
        std::fs::write(path, contents)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;
        Ok(())
    }

    pub fn data_dir(&self) -> PathBuf {
        PathBuf::from(&self.data.directory)
    }

    pub fn data_file(&self) -> PathBuf {
        self.data_dir().join("hours.json")
    }
}

fn expand_tilde(path: &str) -> String {
    shellexpand::tilde(path).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn sample_toml() -> String {
        r#"[data]
directory = "~/Sync/.hours"

[git]
remote = "origin"
auto_push = true

[licensure]
start_date = "2025-01-28"
total_hours_target = 3000
direct_hours_target = 1200
min_months = 24
min_weekly_average = 15.0
"#
        .to_string()
    }

    fn write_config(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("config.toml");
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn load_valid_config() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let path = write_config(tmp.path(), &sample_toml());

        env::remove_var("HOURS_DATA_DIR");
        env::remove_var("HOURS_NO_GIT");

        let config = Config::load_from(&path).unwrap();

        assert!(config.data.directory.contains("Sync/.hours"));
        assert_eq!(config.git.remote, "origin");
        assert!(config.git.auto_push);
        assert_eq!(
            config.licensure.start_date,
            NaiveDate::from_ymd_opt(2025, 1, 28).unwrap()
        );
        assert_eq!(config.licensure.total_hours_target, 3000);
        assert_eq!(config.licensure.direct_hours_target, 1200);
        assert_eq!(config.licensure.min_months, 24);
        assert_eq!(config.licensure.min_weekly_average, 15.0);
    }

    #[test]
    fn load_missing_file_returns_error() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.toml");
        let result = Config::load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn load_malformed_toml_returns_error() {
        let tmp = TempDir::new().unwrap();
        let path = write_config(tmp.path(), "this is not valid toml [[[");
        let result = Config::load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn load_missing_section_returns_error() {
        let tmp = TempDir::new().unwrap();
        let content = r#"[data]
directory = "~/test"
"#;
        let path = write_config(tmp.path(), content);
        let result = Config::load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn env_override_data_dir() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let path = write_config(tmp.path(), &sample_toml());

        let override_dir = tmp.path().join("override_data");
        env::set_var("HOURS_DATA_DIR", override_dir.to_str().unwrap());
        env::remove_var("HOURS_NO_GIT");

        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.data.directory, override_dir.to_str().unwrap());

        env::remove_var("HOURS_DATA_DIR");
    }

    #[test]
    fn env_override_no_git() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let path = write_config(tmp.path(), &sample_toml());

        env::remove_var("HOURS_DATA_DIR");
        env::set_var("HOURS_NO_GIT", "1");

        let config = Config::load_from(&path).unwrap();
        assert!(!config.git.auto_push);

        env::remove_var("HOURS_NO_GIT");
    }

    #[test]
    fn env_no_git_other_values_ignored() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let path = write_config(tmp.path(), &sample_toml());

        env::remove_var("HOURS_DATA_DIR");
        env::set_var("HOURS_NO_GIT", "0");

        let config = Config::load_from(&path).unwrap();
        assert!(config.git.auto_push);

        env::remove_var("HOURS_NO_GIT");
    }

    #[test]
    fn tilde_expansion() {
        let expanded = expand_tilde("~/Sync/.hours");
        assert!(!expanded.starts_with('~'));
        assert!(expanded.contains("Sync/.hours"));
    }

    #[test]
    fn no_tilde_no_change() {
        let expanded = expand_tilde("/absolute/path");
        assert_eq!(expanded, "/absolute/path");
    }

    #[test]
    fn save_and_load_roundtrip() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("subdir").join("config.toml");

        let config = Config {
            data: DataConfig {
                directory: "/tmp/test-data".to_string(),
            },
            git: GitConfig {
                remote: "origin".to_string(),
                auto_push: false,
            },
            licensure: LicensureConfig {
                start_date: NaiveDate::from_ymd_opt(2025, 1, 28).unwrap(),
                total_hours_target: 3000,
                direct_hours_target: 1200,
                min_months: 24,
                min_weekly_average: 15.0,
            },
        };

        config.save(&path).unwrap();
        assert!(path.exists());

        env::remove_var("HOURS_DATA_DIR");
        env::remove_var("HOURS_NO_GIT");

        let loaded = Config::load_from(&path).unwrap();
        assert_eq!(loaded.data.directory, "/tmp/test-data");
        assert_eq!(loaded.git.remote, "origin");
        assert!(!loaded.git.auto_push);
        assert_eq!(loaded.licensure.start_date, config.licensure.start_date);
        assert_eq!(loaded.licensure.total_hours_target, 3000);
        assert_eq!(loaded.licensure.direct_hours_target, 1200);
        assert_eq!(loaded.licensure.min_months, 24);
        assert_eq!(loaded.licensure.min_weekly_average, 15.0);
    }

    #[test]
    fn save_creates_parent_directories() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("a").join("b").join("c").join("config.toml");

        let config = Config {
            data: DataConfig {
                directory: "/tmp/test".to_string(),
            },
            git: GitConfig {
                remote: "origin".to_string(),
                auto_push: true,
            },
            licensure: LicensureConfig {
                start_date: NaiveDate::from_ymd_opt(2025, 1, 28).unwrap(),
                total_hours_target: 3000,
                direct_hours_target: 1200,
                min_months: 24,
                min_weekly_average: 15.0,
            },
        };

        config.save(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn config_dir_uses_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("HOURS_CONFIG_DIR", "/custom/config/path");
        let dir = Config::config_dir();
        assert_eq!(dir, PathBuf::from("/custom/config/path"));
        env::remove_var("HOURS_CONFIG_DIR");
    }

    #[test]
    fn data_dir_and_data_file() {
        let config = Config {
            data: DataConfig {
                directory: "/some/data/dir".to_string(),
            },
            git: GitConfig {
                remote: "origin".to_string(),
                auto_push: true,
            },
            licensure: LicensureConfig {
                start_date: NaiveDate::from_ymd_opt(2025, 1, 28).unwrap(),
                total_hours_target: 3000,
                direct_hours_target: 1200,
                min_months: 24,
                min_weekly_average: 15.0,
            },
        };

        assert_eq!(config.data_dir(), PathBuf::from("/some/data/dir"));
        assert_eq!(
            config.data_file(),
            PathBuf::from("/some/data/dir/hours.json")
        );
    }

    #[test]
    fn load_with_custom_defaults() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let content = r#"[data]
directory = "/custom/path"

[git]
remote = "upstream"
auto_push = false

[licensure]
start_date = "2024-06-04"
total_hours_target = 2000
direct_hours_target = 800
min_months = 12
min_weekly_average = 20.0
"#;
        let path = write_config(tmp.path(), content);

        env::remove_var("HOURS_DATA_DIR");
        env::remove_var("HOURS_NO_GIT");

        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.data.directory, "/custom/path");
        assert_eq!(config.git.remote, "upstream");
        assert!(!config.git.auto_push);
        assert_eq!(
            config.licensure.start_date,
            NaiveDate::from_ymd_opt(2024, 6, 4).unwrap()
        );
        assert_eq!(config.licensure.total_hours_target, 2000);
        assert_eq!(config.licensure.direct_hours_target, 800);
        assert_eq!(config.licensure.min_months, 12);
        assert_eq!(config.licensure.min_weekly_average, 20.0);
    }
}
