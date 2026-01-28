use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use serde::Serialize;
use uuid::Uuid;

pub fn ensure_dir(path: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(path).with_context(|| format!("failed to create dir {}", path.display()))
}

pub fn write_file_atomic(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent for {}", path.display()))?;
    ensure_dir(parent)?;
    let tmp_path = parent.join(format!(".tmp-{}", Uuid::new_v4()));
    {
        let mut file = fs::File::create(&tmp_path)
            .with_context(|| format!("failed to create {}", tmp_path.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("failed to write {}", tmp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("failed to sync {}", tmp_path.display()))?;
    }
    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "failed to rename {} to {}",
            tmp_path.display(),
            path.display()
        )
    })?;
    Ok(())
}

pub fn write_yaml<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let yaml_str = serde_yaml::to_string(value).context("failed to serialize yaml")?;
    write_file_atomic(path, yaml_str.as_bytes())
}
