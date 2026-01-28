use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::de::DeserializeOwned;

pub fn read_yaml<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let contents = fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&contents).context("failed to parse yaml")
}

pub fn read_to_string(path: &Path) -> anyhow::Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}
