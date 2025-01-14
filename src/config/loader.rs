use std::path::Path;
use anyhow::{Result, Context};
use tracing::{info, warn};
use super::Config;

pub fn load_config(path: &Path) -> Result<Config> {
    info!("Loading configuration...");
    info!("Path: {}", path.display());
    info!("Exists: {}", path.exists());
    if !path.exists() {
        info!("No config file found at {}, using default configuration", path.display());
        return Ok(Config::default());
    }

    if path.is_dir() {
        return Err(anyhow::anyhow!("Config path {} is a directory, expected a file", path.display()));
    }

    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    if contents.trim().is_empty() {
        warn!("Config file {} is empty, using default configuration", path.display());
        return Ok(Config::default());
    }

    match path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .as_deref()
    {
        Some("json") => {
            info!("Loading JSON config from {}", path.display());
            serde_json::from_str(&contents)
                .with_context(|| format!("Failed to parse JSON config from {}", path.display()))
        }
        Some("yaml") | Some("yml") => {
            info!("Loading YAML config from {}", path.display());
            serde_yaml::from_str(&contents)
                .with_context(|| format!("Failed to parse YAML config from {}", path.display()))
        }
        Some(ext) => {
            Err(anyhow::anyhow!("Unsupported config file format: .{}", ext))
        }
        None => {
            Err(anyhow::anyhow!("Config file {} has no extension", path.display()))
        }
    }
} 