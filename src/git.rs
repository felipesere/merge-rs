use std::process::Command;

use anyhow::Context;
use time::{format_description, OffsetDateTime};

pub fn current_git_sha() -> Result<String, anyhow::Error> {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .context("failed to get current git")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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
