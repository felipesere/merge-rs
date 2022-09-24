use std::process::Command;

use anyhow::{anyhow, Context};
use time::{format_description, OffsetDateTime};

pub fn current_git_sha() -> Result<String, anyhow::Error> {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .context("failed to get current git")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn current_branch() -> Result<String, anyhow::Error> {
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("failed to get current git")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn restore() -> Result<(), anyhow::Error> {
    let o = Command::new("git")
        .args(&["restore", "."])
        .status()
        .context("failed to get current git")?;

    if o.success() {
        Ok(())
    } else {
        anyhow::bail!("Failed to do simple merge")
    }
}

pub fn switch_branch(branch: &str) -> Result<(), anyhow::Error> {
    let o = Command::new("git")
        .args(&["switch", branch])
        .status()
        .context("failed to get current git")?;

    if o.success() {
        Ok(())
    } else {
        anyhow::bail!("Failed to do simple merge")
    }
}

pub fn create_new_branch() -> Result<(), anyhow::Error> {
    let format = format_description::parse("[year]-[month]-[day]")?;
    let today = OffsetDateTime::now_utc()
        .format(&format)
        .context("Couldn't get current time")?;
    Command::new("git")
        .args(&["switch", "-c"])
        .arg(format!("renovate-{today}"))
        .status()
        .context("failed to get current git")?;

    Ok(())
}

pub fn reset_hard(sha: &str) -> Result<(), anyhow::Error> {
    let o = Command::new("git")
        .args(&["reset", "--hard", sha])
        .status()
        .context("failed to get current git")?;

    if o.success() {
        Ok(())
    } else {
        anyhow::bail!("Failed to do simple merge")
    }
}

pub fn simple_merge(pr: &str) -> Result<(), anyhow::Error> {
    let o = Command::new("git")
        .args(&["merge", "--quiet", "--no-edit"])
        .arg(format!("origin/{pr}"))
        .status()
        .context("failed to get current git")?;

    if o.code() == Some(0) {
        Ok(())
    } else {
        anyhow::bail!("Failed to do simple merge")
    }
}

pub fn abort_merge() -> Result<(), anyhow::Error> {
    let o = Command::new("git")
        .args(&["merge", "--abort"])
        .status()
        .context("failed to get current git")?;

    if o.code() == Some(0) {
        Ok(())
    } else {
        anyhow::bail!("Failed to abort merge")
    }
}
