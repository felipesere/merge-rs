use crate::{git, state};

pub fn run() -> Result<(), anyhow::Error> {
    let state = state::load()?;
    git::restore()?;
    git::switch_branch(&state.starting_branch)?;
    git::reset_hard(&state.starting_git_sha)?;

    state::delete()?;
    Ok(())
}
