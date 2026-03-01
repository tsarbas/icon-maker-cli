use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfig {
    pub openai_key: Option<String>,
    pub model: Option<String>,
}

pub fn load() -> Result<Option<AppConfig>> {
    match default_path() {
        Some(path) => load_from_path(&path),
        None => Ok(None),
    }
}

fn load_from_path(path: &Path) -> Result<Option<AppConfig>> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(err).with_context(|| format!("failed to read {}", path.display()));
        }
    };

    let parsed: AppConfig = toml::from_str(&raw)
        .with_context(|| format!("failed to parse TOML config at {}", path.display()))?;
    Ok(Some(parsed))
}

fn default_path() -> Option<PathBuf> {
    if let Some(base) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(base).join("icon-maker").join("config.toml"));
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config").join("icon-maker").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::load_from_path;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(file: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("icon-maker-config-{ts}-{file}"))
    }

    #[test]
    fn load_from_path_returns_none_when_missing() {
        let path = temp_path("missing.toml");
        let loaded = load_from_path(&path).expect("load missing file");
        assert!(loaded.is_none());
    }

    #[test]
    fn load_from_path_parses_values() {
        let path = temp_path("valid.toml");
        fs::write(
            &path,
            "openai_key = \"sk-test\"\nmodel = \"gpt-image-1.5\"\n",
        )
        .expect("write config");

        let loaded = load_from_path(&path).expect("load config").expect("config exists");
        assert_eq!(loaded.openai_key.as_deref(), Some("sk-test"));
        assert_eq!(loaded.model.as_deref(), Some("gpt-image-1.5"));

        fs::remove_file(path).expect("cleanup");
    }

    #[test]
    fn load_from_path_fails_on_invalid_toml() {
        let path = temp_path("invalid.toml");
        fs::write(&path, "openai_key = [}").expect("write bad config");

        let err = load_from_path(&path).expect_err("expected parse error");
        assert!(
            err.to_string().contains("failed to parse TOML config"),
            "unexpected error: {err}"
        );

        fs::remove_file(path).expect("cleanup");
    }
}
