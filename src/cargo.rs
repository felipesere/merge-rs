use std::process::Command;

use anyhow::Context;

pub fn build() -> Result<(), anyhow::Error> {
    let o = Command::new("cargo")
        .args(&["build"])
        .status()
        .context("failed to get current git")?;

    if o.code() == Some(0) {
        Ok(())
    } else {
        anyhow::bail!("Failed to delete branch")
    }
}
