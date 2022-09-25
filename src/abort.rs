use crate::{git, state};

pub fn run() -> Result<(), anyhow::Error> {
    let state = state::load()?;
    let current_branch = git::current_branch()?;
    git::restore()?;
    git::switch_branch(&state.starting_branch)?;
    git::delete_branch(&current_branch)?;
    git::reset_hard(&state.starting_git_sha)?;

    state::delete()?;
    Ok(())
}
