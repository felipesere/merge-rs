use std::process::Command;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{git, cargo};

pub fn run() -> Result<(), anyhow::Error> {
    let prs = get_renovate_prs()?;
    let current_sha = git::current_git_sha()?;
    let state =
        crate::state::initial_state(prs, current_sha).context("Could not great initial state")?;
    git::create_new_branch()?;

    for pr in state.prs.possible_prs {
        let copy = pr.clone();
        crate::state::update_state(|mut state| {
            state.current_pr = Some(copy);
            state
        })?;
        let outcome = git::simple_merge(&pr);
        if outcome.is_err() {
            if git::try_mergetool().is_ok() {
                for lock_file in glob::glob("**/Cargo.lock").expect("Failed to read the glob pattern") {
                    let lock_file = lock_file.expect("To have a file");
                    if lock_file.is_file() {
                        git::checkout_theirs(&lock_file)?;
                        git::add(&lock_file)?;
                    }
                }
                git::merge_continue()?;
                cargo::build()?;
            } else {
                git::abort_merge()?;
                crate::state::update_state(|mut state| {
                    state.failed_to_merge.push(pr.clone());
                    state
                })?;
                continue;
            }
        }
        crate::state::update_state(|mut state| {
            state.succeeded_to_merge.push(pr.clone());
            state
        })?;
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pr {
    author: Author,
    head_ref_name: String,
    status_check_rollup: Vec<Check>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Author {
    login: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Check {
    state: Option<CiState>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum CiState {
    Failure,
    Completed,
    Success,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Prs {
    pub ci_failures: Vec<String>,
    pub possible_prs: Vec<String>,
}

fn get_renovate_prs() -> Result<Prs, anyhow::Error> {
    let x = Command::new("gh")
        .args(&[
            "pr",
            "list",
            "--json",
            "author,headRefName,statusCheckRollup",
        ])
        .output()
        .context("failed to get GH prs for current repo")?;

    let prs: Vec<Pr> = serde_json::from_reader(x.stdout.as_slice())?;
    let (ci_failures, possible_prs): (Vec<_>, Vec<_>) = prs
        .into_iter()
        .filter(|pr| pr.author.login == "tl-admins")
        .partition(|pr| {
            pr.status_check_rollup
                .iter()
                .any(|s| s.state == Some(CiState::Failure))
        });

    let ci_failures: Vec<_> = ci_failures.into_iter().map(|pr| pr.head_ref_name).collect();
    let possible_prs: Vec<_> = possible_prs
        .into_iter()
        .map(|pr| pr.head_ref_name)
        .collect();

    Ok(Prs {
        ci_failures,
        possible_prs,
    })
}
