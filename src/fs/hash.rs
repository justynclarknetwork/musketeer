use std::fs;
use std::path::Path;

use anyhow::Context;
use sha2::{Digest, Sha256};

pub fn hash_file_hex(path: &Path) -> anyhow::Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}
