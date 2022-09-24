use std::process::Command;

use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn run() -> Result<(), anyhow::Error> {
    let prs = get_renovate_prs()?;
    dbg!(&prs);
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

#[derive(Debug)]
struct Prs {
    ci_failures: Vec<String>,
    possible_prs: Vec<String>,
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
