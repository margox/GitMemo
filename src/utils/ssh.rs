use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Generate an ED25519 SSH key pair (skip if already exists)
pub fn generate_key(key_dir: &Path) -> Result<(PathBuf, bool)> {
    std::fs::create_dir_all(key_dir)?;

    let key_path = key_dir.join("id_ed25519");

    if key_path.exists() {
        return Ok((key_path, false)); // already exists, not newly generated
    }

    let status = Command::new("ssh-keygen")
        .args([
            "-t", "ed25519",
            "-f", key_path.to_str().unwrap(),
            "-N", "",
            "-C", "gitmemo",
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("ssh-keygen failed");
    }

    Ok((key_path, true)) // newly generated
}

/// Read the public key
pub fn read_public_key(key_path: &Path) -> Result<String> {
    let pub_path = key_path.with_extension("pub");
    Ok(std::fs::read_to_string(pub_path)?.trim().to_string())
}
