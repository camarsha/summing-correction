mod efficiency;
mod level_info;
mod read_levels;
mod sum_correction;
use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use indicatif::ProgressBar;
use level_info::Observation;
use rgsl::MatrixF64;
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
/// Correct gamma ray intensities based on the summing correction
/// formalism of Semkov.
struct Args {
    /// Input file with branching level, branching ratios, and observed values.
    input: String,

    /// Path to the two column peak efficiency file.
    #[arg(short, long)]
    peak_eff_file: Option<String>,

    /// Path to the two column total efficiency file.
    #[arg(short, long)]
    total_eff_file: Option<String>,

    /// Number of Monte-Carlo samples to run.
    #[arg(short, long, default_value_t = 10000)]
    samples: i64,

    /// Output file
    #[arg(short, long)]
    output: Option<String>,

    /// Output formatted for humans
    #[arg(short, long, default_value_t = false)]
    readable: bool,

    /// Just a single run, no sampling
    #[arg(short, long, default_value_t = false)]
    no_samples: bool,

    /// Observed counts for a single transition. <inital state> <final state> <counts> <dcounts>
    #[arg(short, long, num_args = 4, value_delimiter = ' ')]
    counts: Option<Vec<f64>>,
}

fn print_function(
    obs: &mut [Observation],
    energy_matrix: &MatrixF64,
    in_file: &str,
    for_humans: bool,
) {
    if for_humans {
        for o in obs.iter_mut() {
            match o.corrected_value() {
                Ok((m, std)) => println!(
                    "E𝛾 = {0:<10.2} | Observed = {1:<7.1} ± {2:<5.1} | Corrected = {m:<7.1} ± {std:<5.1}",
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
    } else {
        println!("Eg,counts,dcounts,corrected,dcorrected");
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
}

fn write_output(obs: &mut [Observation], energy_matrix: &MatrixF64, in_file: &str, out_file: &str) {
    let output = File::create(out_file).expect("Failed to create output file!");
    let mut buf_writer = BufWriter::new(output);
    writeln!(buf_writer, "Eg,counts,dcounts,corrected,dcorrected")
        .expect("Failed to write csv header!");
    for o in obs.iter_mut() {
        match o.corrected_value() {
            Ok((m, std)) => writeln!(
                buf_writer,
                "{0:.2},{1:.3},{2:.3},{m:.3},{std:.3}",
                energy_matrix.get(o.from, o.to),
                o.counts,
                o.dcounts
            )
            .expect("Data write failed!"),
            Err(()) => {
                eprintln!(
                    "Observed transition from {} to {} was not defined in the B-Values section of {}, skipping!",
                    o.from, o.to, in_file
                );
            }
        };
    }
}

fn main() -> Result<()> {
    // Better panic messages.
    color_eyre::install()?;

    let mut r = rand::rng();
    let args = Args::parse();

    let in_file = args.input;

    let peak_file = match args.peak_eff_file {
        Some(p) => p,
        None => "peak_eff.dat".to_string(),
    };

    let total_file = match args.total_eff_file {
        Some(p) => p,
        None => "tot_eff.dat".to_string(),
    };

    let n_samples = if args.no_samples || args.samples < 1 {
        1
    } else {
        args.samples as usize
    };

    let (levels, branches, mut obs) = read_levels::read_input(&in_file, n_samples);

    // Now if the user gave a count input we add it to the obs
    if let Some(counts) = args.counts {
        if counts[0].fract() != 0.0 {
            return Err(eyre!("First argument to --counts must be an integer!"));
        }
        if counts[1].fract() != 0.0 {
            return Err(eyre!("Second argument to --counts must be an integer!"));
        }
        obs.push(Observation::new(
            counts[0] as usize,
            counts[1] as usize,
            counts[2],
            counts[3],
            n_samples,
        ));
    }

    let mut peak_eff_spline = efficiency::make_efficiency(&peak_file);
    let mut total_eff_spline = efficiency::make_efficiency(&total_file);

    if args.no_samples || n_samples <= 1 {
        let (x, f) = sum_correction::make_x_and_f_matrix(&branches, &levels);
        let energy_matrix = sum_correction::make_transition_energies(&branches, &levels);
        let (peak_matrix, total_matrix) = sum_correction::make_eff_matrix(
            &energy_matrix,
            &mut peak_eff_spline,
            &mut total_eff_spline,
        );
        let correction = sum_correction::calculate_correction(&x, &f, &peak_matrix, &total_matrix);
        for o in obs.iter_mut() {
            o.add_correction(0, &correction);
        }
    } else {
        let bar = ProgressBar::new(n_samples as u64);
        for i in 0..n_samples {
            bar.inc(1);
            let temp_level: Vec<level_info::Level> =
                levels.iter().map(|l| l.sample(&mut r)).collect();
            let temp_branch: Vec<level_info::Branch> =
                branches.iter().map(|b| b.sample(&mut r)).collect();

            let (x, f) = sum_correction::make_x_and_f_matrix(&temp_branch, &temp_level);
            let energy_matrix = sum_correction::make_transition_energies(&temp_branch, &temp_level);
            let (peak_matrix, total_matrix) = sum_correction::make_eff_matrix(
                &energy_matrix,
                &mut peak_eff_spline,
                &mut total_eff_spline,
            );

            let correction =
                sum_correction::calculate_correction(&x, &f, &peak_matrix, &total_matrix);
            for o in obs.iter_mut() {
                o.add_correction(i, &correction);
            }
        }
        bar.finish();
    }
    let energy_matrix = sum_correction::make_transition_energies(&branches, &levels);

    if let Some(out_file) = args.output {
        // You can still see nice output if you want it.
        if args.readable {
            print_function(&mut obs, &energy_matrix, &in_file, args.readable);
        }
        write_output(&mut obs, &energy_matrix, &in_file, &out_file);
    } else {
        print_function(&mut obs, &energy_matrix, &in_file, args.readable);
    }

    Ok(())
}
