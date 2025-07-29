mod draw_levels;
mod efficiency;
mod level_info;
mod read_levels;
mod sum_correction;

fn main() {
    // Hard coded file paths for now
    let in_file = "/home/caleb/Research/UNC/Code/Sum-Correction/example-input/22Ne.dat";
    let peak_file = "/home/caleb/Research/UNC/Code/Sum-Correction/example-input/peak_eff.dat";
    let total_file = "/home/caleb/Research/UNC/Code/Sum-Correction/example-input/tot_eff.dat";

    let (levels, branches, obs) = read_levels::read_input(in_file);
    let mut peak_eff_spline = efficiency::make_efficiency(peak_file);
    let mut total_eff_spline = efficiency::make_efficiency(total_file);

    // draw_levels::print_terminal_diagram(&levels, &branches);
    let (x, f) = sum_correction::make_x_and_f_matrix(&branches, &levels);
    let energy_matrix = sum_correction::make_tranition_energies(&branches, &levels);
    let (peak_matrix, total_matrix) = sum_correction::make_eff_matrix(
        &energy_matrix,
        &mut peak_eff_spline,
        &mut total_eff_spline,
    );

    let correction = sum_correction::calculate_correction(&x, &f, &peak_matrix, &total_matrix);
    for o in obs.iter() {
        sum_correction::correct(o.clone(), &correction);
    }
}
