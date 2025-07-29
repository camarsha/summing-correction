use crate::level_info::{Branch, Level};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/// This is all gpt crap.

pub fn write_dot_file(
    levels: &[Level],
    transitions: &[Branch],
    filename: &str,
) -> std::io::Result<()> {
    let mut file = File::create(filename)?;

    writeln!(file, "digraph G {{")?;
    writeln!(file, "  rankdir=TB;")?; // levels from top (high E) to bottom

    // Define level nodes with energy labels
    for level in levels {
        writeln!(
            file,
            "  {} [label=\"{}\\n{:.1} keV\", shape=ellipse];",
            level.idx, level.idx, level.energy
        )?;
    }

    // Add transitions as directed edges
    let energy_map: HashMap<_, _> = levels
        .iter()
        .map(|lvl| (lvl.idx.clone(), lvl.energy))
        .collect();

    for trans in transitions {
        if let (Some(&e1), Some(&e2)) = (energy_map.get(&trans.from), energy_map.get(&trans.to)) {
            let e_gamma = e1 - e2;
            writeln!(
                file,
                "  {} -> {} [label=\"{:.1} keV\\nI={:.2}\"];",
                trans.from, trans.to, e_gamma, trans.val
            )?;
        }
    }

    writeln!(file, "}}")?;
    Ok(())
}

pub fn print_terminal_diagram(levels: &[Level], transitions: &[Branch]) {
    // Sort levels by descending energy for top-down display
    let mut sorted_levels = levels.to_vec();
    sorted_levels.sort_by(|a, b| b.energy.partial_cmp(&a.energy).unwrap());

    // Map for easy access to transitions from each level
    let mut transition_map: HashMap<usize, Vec<(usize, f64)>> = HashMap::new();
    let level_map: HashMap<_, _> = levels.iter().map(|lvl| (lvl.idx, lvl.energy)).collect();

    for trans in transitions {
        if let (Some(&e_from), Some(&e_to)) = (level_map.get(&trans.from), level_map.get(&trans.to))
        {
            let e_gamma = e_from - e_to;
            transition_map
                .entry(trans.from)
                .or_default()
                .push((trans.to, e_gamma));
        }
    }

    // Print each level and transitions below it
    for level in &sorted_levels {
        println!("{:>7.1} keV ── [{}]", level.energy, level.idx);
        let key = level.idx;
        if let Some(edges) = transition_map.get(&key) {
            for (target, e_gamma) in edges {
                println!("             │");
                println!("             ▼ {:.1} keV ", e_gamma);
            }
        }
    }
}
