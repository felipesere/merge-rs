use crate::state;

pub fn run() -> Result<(), anyhow::Error> {
    let state = state::load()?;

    println!(
        "Current PR: {}",
        state.current_pr.as_deref().unwrap_or("NONE")
    );

    println!("Successful merges:");
    for pr in &state.succeeded_to_merge {
        println!("\t{pr}");
    }
    println!("Failed to merge");
    for pr in &state.failed_to_merge {
        println!("\t{pr}");
    }

    Ok(())
}
