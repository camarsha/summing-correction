use crate::level_info::{Branch, Level, Observation};
/// This module handles the user input file.
/// The input file is expected to be in the traditional LENA style
/// You should have the following sections Energy-Levels, B-Values, and Observed-Values
use std::fs;

#[derive(Debug)]
enum FileSection {
    None,
    EnergyLevels,
    BValues,
    ObservedValues,
}

fn parse_header(line: &str) -> FileSection {
    match line {
        "Energy-Levels" => FileSection::EnergyLevels,
        "B-Values" => FileSection::BValues,
        "Observed-Values" => FileSection::ObservedValues,
        _ => panic!("{line} is not a valid section header!"),
    }
}

fn parse_energy(line: &str, idx: i32) -> Level {
    let mut parts = line.split_whitespace();
    let energy: f64 = parts
        .next()
        .expect("Malformed Energy Line: {line}\n")
        .parse()
        .expect("Unable to parse level energy!\n");
    let feeding: f64 = parts
        .next()
        .expect("Malformed Energy Line: {line}\n")
        .parse()
        .expect("Unable to parse feeding fraction!\n");
    let dfeeding: f64 = parts
        .next()
        .expect("Malformed Energy Line: {line}\n")
        .parse()
        .expect("Unable to parse feeding fraction uncertainty!\n");
    Level::new(idx as usize, energy, 0.0, feeding, dfeeding)
}

fn parse_branch(line: &str) -> Branch {
    let mut parts = line.split_whitespace();
    let from: usize = parts
        .next()
        .expect("Malformed Branch Line: {line}\n")
        .parse()
        .expect("Unable to parse branch from!\n");
    let to: usize = parts
        .next()
        .expect("Malformed Energy Line: {line}\n")
        .parse()
        .expect("Unable to parse branch to!\n");
    let val: f64 = parts
        .next()
        .expect("Malformed Energy Line: {line}\n")
        .parse()
        .expect("Unable to parse branch intensity!\n");
    let dval: f64 = parts
        .next()
        .expect("Malformed Energy Line: {line}\n")
        .parse()
        .expect("Unable to parse branch intensity uncertainty!\n");

    Branch::new(from, to, val, dval)
}

fn parse_obs(line: &str) -> Observation {
    let mut parts = line.split_whitespace();
    let from: usize = parts
        .next()
        .expect("Malformed Observation Line: {line}\n")
        .parse()
        .expect("Unable to parse observation from!\n");
    let to: usize = parts
        .next()
        .expect("Malformed Observation Line: {line}\n")
        .parse()
        .expect("Unable to parse observation to!\n");
    let counts: f64 = parts
        .next()
        .expect("Malformed Observation Line: {line}\n")
        .parse()
        .expect("Unable to parse observation counts!\n");
    let dcounts: f64 = parts
        .next()
        .expect("Malformed Observation Line: {line}\n")
        .parse()
        .expect("Unable to parse observation counts uncertainty!\n");

    Observation::new(from, to, counts, dcounts)
}

pub fn read_input(file_path: &str) -> (Vec<Level>, Vec<Branch>, Vec<Observation>) {
    let file_content =
        fs::read_to_string(file_path).expect(format!("Failed to read: {file_path}\n").as_str());
    let mut current_section = FileSection::None;
    let mut levels: Vec<Level> = Vec::new();
    let mut branchs: Vec<Branch> = Vec::new();
    let mut obs: Vec<Observation> = Vec::new();
    let mut idx = 0;
    let mut counter = || {
        idx += 1;
        idx - 1
    };
    for line in file_content.lines() {
        let trimmed = line.trim();
        if line.is_empty() {
            current_section = FileSection::None;
            continue;
        }
        match current_section {
            FileSection::None => current_section = parse_header(trimmed),
            FileSection::EnergyLevels => levels.push(parse_energy(trimmed, counter())),
            FileSection::BValues => branchs.push(parse_branch(trimmed)),
            FileSection::ObservedValues => obs.push(parse_obs(trimmed)),
        }
    }
    (levels, branchs, obs)
}
