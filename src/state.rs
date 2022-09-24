use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::git;
use crate::start::Prs;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub starting_git_sha: String,
    pub starting_branch: String,
    pub prs: Prs,
    pub current_pr: Option<String>,
    pub failed_to_merge: Vec<String>,
    pub succeeded_to_merge: Vec<String>,
}

pub fn initial_state(prs: Prs, sha: String) -> Result<State, anyhow::Error> {
    let branch = git::current_branch()?;
    let state = State {
        starting_git_sha: sha,
        starting_branch: branch,
        prs,
        current_pr: None,
        failed_to_merge: vec![],
        succeeded_to_merge: vec![],
    };

    let contents = serde_json::to_string_pretty(&state).context("Could not create state")?;
    std::fs::write(".renovate-merge-state.json", contents).context("Could not write state")?;

    Ok(state)
}
pub fn load() -> Result<State, anyhow::Error> {
    let content = std::fs::read_to_string(".renovate-merge-state.json")
        .context("Could not read state file")?;
    serde_json::from_str(&content).context("Could not load state")
}

pub fn update_state<F: FnOnce(State) -> State>(func: F) -> Result<State, anyhow::Error> {
    let current_state = load()?;

    let next = func(current_state);
    let contents = serde_json::to_string_pretty(&next).context("Could not create state")?;
    std::fs::write(".renovate-merge-state.json", contents).context("Could not write state")?;

    Ok(next)
}

pub fn delete() -> Result<(), anyhow::Error> {
    std::fs::remove_file(".renovate-merge-state.json").context("Failed to delete state file")?;
    Ok(())
}
