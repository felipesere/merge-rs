use crate::state;

pub fn run() -> Result<(), anyhow::Error> {
    let s = state::load()?;

    let current_pr = if let Some(pr) = s.current_pr {
        pr
    } else {
        anyhow::bail!("There was no current pr to continue from");
    };
    let outstanding_prs: Vec<String> = s
        .prs
        .possible_prs
        .iter()
        .cloned()
        .skip_while(|pr| **pr != current_pr)
        .collect();

    crate::start::merge_prs(&outstanding_prs)?;

    Ok(())
}
