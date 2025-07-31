// mod draw_levels;
mod efficiency;
mod interpreter;
mod level_info;
mod read_levels;
mod sum_correction;
use clap::Parser;
use indicatif::ProgressBar;

use std::{
    env,
    path::{self, Path},
    process::exit,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Correct gamma ray intensities based on the summing correction
/// formalism of Semkov.
struct Args {
    /// Input file with branching level, branching ratios, and observed values.
    input: Option<String>,

    /// Path to the two column peak efficiency file.
    #[arg(short, long)]
    peak_eff_file: Option<String>,

    /// Path to the two column total efficiency file.
    #[arg(short, long)]
    total_eff_file: Option<String>,

    #[arg(short, long, default_value_t = 10000)]
    samples: i64,
}

fn main() {
    // First lets keep around some paths for convenience.
    // let home_dir = env::home_dir().expect("Failed to locate home directory.");
    // let current_dir = env::current_dir().expect("Failed to locate current directory.");
    let mut r = rand::rng();
    let args = Args::parse();

    // if args.input.is_none() {
    //     interpreter::read_loop().unwrap();
    //     return;
    // }

    let in_file = match args.input {
        Some(p) => p,
        None => "".to_string(),
    };

    let peak_file = match args.peak_eff_file {
        Some(p) => p,
        None => "peak_eff.dat".to_string(),
    };

    let total_file = match args.total_eff_file {
        Some(p) => p,
        None => "tot_eff.dat".to_string(),
    };

    let n_samples = args.samples as usize;

    let bar = ProgressBar::new(n_samples as u64);

    let (levels, branches, mut obs) = read_levels::read_input(&in_file, n_samples);
    let mut peak_eff_spline = efficiency::make_efficiency(&peak_file);
    let mut total_eff_spline = efficiency::make_efficiency(&total_file);

    for i in 0..n_samples {
        bar.inc(1);
        let temp_level: Vec<level_info::Level> = levels.iter().map(|l| l.sample(&mut r)).collect();
        let temp_branch: Vec<level_info::Branch> =
            branches.iter().map(|b| b.sample(&mut r)).collect();

        let (x, f) = sum_correction::make_x_and_f_matrix(&temp_branch, &temp_level);
        let energy_matrix = sum_correction::make_tranition_energies(&temp_branch, &temp_level);
        let (peak_matrix, total_matrix) = sum_correction::make_eff_matrix(
            &energy_matrix,
            &mut peak_eff_spline,
            &mut total_eff_spline,
        );

        let correction = sum_correction::calculate_correction(&x, &f, &peak_matrix, &total_matrix);
        for o in obs.iter_mut() {
            o.add_correction(i, &correction);
        }
    }
    bar.finish();

    let energy_matrix = sum_correction::make_tranition_energies(&branches, &levels);

    println!("E𝛾,counts,dcounts,corrected,dcorrected");
    for o in obs.iter_mut() {
        match o.corrected_value() {
            Ok((m, std)) => println!(
                "{0:.2},{1:.3},{2:.3},{m:.3},{std:.3}",
                energy_matrix.get(o.from, o.to),
                o.counts,
                o.dcounts
            ),
            Err(()) => eprintln!(
                "Observed transition from {} to {} was not defined in the B-Values section of {}, skipping!",
                o.from, o.to, in_file
            ),
        };
    }
}
