use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Known SSH key filenames to search for (in priority order)
const KEY_NAMES: &[&str] = &["id_ed25519", "id_rsa", "id_ecdsa"];

/// Find an existing SSH private key in ~/.ssh/
pub fn find_existing_key() -> Option<PathBuf> {
    let ssh_dir = dirs::home_dir()?.join(".ssh");
    if !ssh_dir.is_dir() {
        return None;
    }
    for name in KEY_NAMES {
        let key = ssh_dir.join(name);
        if key.exists() {
            return Some(key);
        }
    }
    None
}

/// Find or generate an SSH key.
///
/// Strategy:
/// 1. Check ~/.ssh/ for existing keys — reuse if found
/// 2. Check gitmemo's own key dir (~/.gitmemo/.ssh/) — reuse if found
/// 3. Generate a new key in the gitmemo key dir
///
/// Returns (key_path, is_new_key, is_system_key)
pub fn find_or_generate_key(gitmemo_ssh_dir: &Path) -> Result<(PathBuf, bool, bool)> {
    // 1. Check system ~/.ssh/
    if let Some(existing) = find_existing_key() {
        return Ok((existing, false, true));
    }

    // 2. Check gitmemo's own .ssh dir
    std::fs::create_dir_all(gitmemo_ssh_dir)?;
    let key_path = gitmemo_ssh_dir.join("id_ed25519");
    if key_path.exists() {
        return Ok((key_path, false, false));
    }

    // 3. Generate new key in gitmemo dir
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

    Ok((key_path, true, false))
}

/// Read the public key content
pub fn read_public_key(key_path: &Path) -> Result<String> {
    let pub_path = key_path.with_extension("pub");
    Ok(std::fs::read_to_string(pub_path)?.trim().to_string())
}

/// Build GIT_SSH_COMMAND value that uses the given private key.
/// Returns None if the key is in ~/.ssh/ (system default, no override needed).
pub fn git_ssh_command(key_path: &Path) -> Option<String> {
    let home_ssh = dirs::home_dir().map(|h| h.join(".ssh"));
    if let Some(ref ssh_dir) = home_ssh {
        if key_path.starts_with(ssh_dir) {
            return None; // system default, git will find it automatically
        }
    }
    Some(format!(
        "ssh -i {} -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new",
        key_path.display()
    ))
}

/// Extract the SSH host from a git URL (e.g. "git@github.com:user/repo.git" -> "github.com")
fn extract_ssh_host(url: &str) -> Option<String> {
    // SSH format: git@github.com:user/repo.git
    if let Some(at_pos) = url.find('@') {
        let after_at = &url[at_pos + 1..];
        if let Some(colon_pos) = after_at.find(':') {
            return Some(after_at[..colon_pos].to_string());
        }
    }
    // ssh://git@github.com/user/repo.git
    if url.starts_with("ssh://") {
        let after_scheme = &url[6..]; // skip "ssh://"
        if let Some(at_pos) = after_scheme.find('@') {
            let after_at = &after_scheme[at_pos + 1..];
            // host is until first '/' or ':'
            let host_end = after_at.find('/').or_else(|| after_at.find(':'));
            if let Some(end) = host_end {
                return Some(after_at[..end].to_string());
            }
        }
    }
    None
}

/// Test SSH connection to a git host.
/// Returns Ok(true) on success, Ok(false) on auth failure, Err on other errors.
pub fn test_ssh_connection(key_path: &Path, git_url: &str) -> Result<SshTestResult> {
    let host = match extract_ssh_host(git_url) {
        Some(h) => h,
        None => return Ok(SshTestResult::NotSsh),
    };

    let mut cmd = Command::new("ssh");
    cmd.args(["-T", &format!("git@{}", host)])
        .args(["-o", "StrictHostKeyChecking=accept-new"])
        .args(["-o", "ConnectTimeout=10"]);

    // Use specific key if not in system ~/.ssh/
    if let Some(ssh_cmd) = git_ssh_command(key_path) {
        // Parse the -i argument from the ssh command
        cmd.args(["-i", key_path.to_str().unwrap()])
            .args(["-o", "IdentitiesOnly=yes"]);
        let _ = ssh_cmd; // used for the logic above
    }

    let output = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let combined = format!("{} {}", stdout, stderr);

    // GitHub/GitLab return exit code 1 but print "successfully authenticated"
    if combined.contains("successfully authenticated")
        || combined.contains("Welcome to GitLab")
        || combined.contains("Hi ")
        || combined.contains("welcome")
    {
        return Ok(SshTestResult::Success(stderr.trim().to_string()));
    }

    if combined.contains("Permission denied") || combined.contains("publickey") {
        return Ok(SshTestResult::AuthFailed(stderr.trim().to_string()));
    }

    if combined.contains("Connection refused")
        || combined.contains("Connection timed out")
        || combined.contains("Could not resolve hostname")
    {
        return Ok(SshTestResult::ConnectionFailed(stderr.trim().to_string()));
    }

    // Unknown result — could be success on some platforms
    if output.status.code() == Some(1) {
        // Many git hosts return 1 on ssh -T but it means success
        return Ok(SshTestResult::Success(stderr.trim().to_string()));
    }

    Ok(SshTestResult::Unknown(stderr.trim().to_string()))
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum SshTestResult {
    /// SSH connection authenticated successfully
    Success(String),
    /// Not an SSH URL (HTTPS etc.)
    NotSsh,
    /// Authentication failed (key not recognized)
    AuthFailed(String),
    /// Network-level connection failure
    ConnectionFailed(String),
    /// Unknown result
    Unknown(String),
}

/// Check if a git URL is SSH-based
pub fn is_ssh_url(url: &str) -> bool {
    url.starts_with("git@") || url.starts_with("ssh://")
}

/// Convert HTTPS GitHub/GitLab URL to SSH format
pub fn https_to_ssh(url: &str) -> Option<String> {
    // https://github.com/user/repo.git -> git@github.com:user/repo.git
    // https://gitlab.com/user/repo.git -> git@gitlab.com:user/repo.git
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return None;
    }
    let stripped = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let parts: Vec<&str> = stripped.splitn(2, '/').collect();
    if parts.len() != 2 {
        return None;
    }
    let host = parts[0];
    let path = parts[1];
    Some(format!("git@{}:{}", host, path))
}
