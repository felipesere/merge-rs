use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::start::Prs;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub starting_git_sha: String,
    pub prs: Prs,
    pub current_pr: Option<String>,
    pub failed_to_merge: Vec<String>,
    pub succeeded_to_merge: Vec<String>,
}

pub fn initial_state(prs: Prs, sha: String) -> Result<State, anyhow::Error> {
    let s = State {
        starting_git_sha: sha,
        prs,
        current_pr: None,
        failed_to_merge: vec![],
        succeeded_to_merge: vec![],
    };

    let contents = serde_json::to_string_pretty(&s).context("Could not create state")?;
    std::fs::write(".renovate-merge-state.json", contents).context("Could not write state")?;

    Ok(s)
}
