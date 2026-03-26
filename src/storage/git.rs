use anyhow::Result;
use std::path::Path;

/// Initialize or open Git repository at the given path
pub fn init_repo(repo_path: &Path, remote_url: &str) -> Result<git2::Repository> {
    let repo = if repo_path.join(".git").exists() {
        // Open existing repo
        let repo = git2::Repository::open(repo_path)?;
        // Update remote URL if different
        if let Ok(remote) = repo.find_remote("origin") {
            if remote.url().unwrap_or("") != remote_url {
                drop(remote);
                repo.remote_set_url("origin", remote_url)?;
            }
        } else {
            repo.remote("origin", remote_url)?;
        }
        repo
    } else {
        // Create new repo
        let repo = git2::Repository::init(repo_path)?;
        repo.remote("origin", remote_url)?;
        repo
    };

    Ok(repo)
}

/// Stage all changes, commit, and push
pub fn commit_and_push(repo_path: &Path, message: &str) -> Result<()> {
    let repo = git2::Repository::open(repo_path)?;

    // Stage all
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    // Check if there's anything to commit
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let sig = git2::Signature::now("GitMemo", "bot@gitmemo.dev")?;

    if let Ok(head) = repo.head() {
        let parent = head.peel_to_commit()?;
        // Skip if tree unchanged
        if parent.tree()?.id() == tree_id {
            return Ok(());
        }
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?;
    } else {
        // Initial commit
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?;
    }

    // Push using system git (handles SSH auth via ssh-agent / system keychain)
    let _ = std::process::Command::new("git")
        .args(["push", "origin", "HEAD"])
        .current_dir(repo_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    Ok(())
}

/// Test if remote is reachable
#[allow(dead_code)]
pub fn test_remote(repo_path: &Path) -> Result<()> {
    let repo = git2::Repository::open(repo_path)?;
    let mut remote = repo.find_remote("origin")?;
    remote.connect(git2::Direction::Fetch)?;
    remote.disconnect()?;
    Ok(())
}
