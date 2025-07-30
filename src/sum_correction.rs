use crate::{
    efficiency::Efficiency,
    level_info::{Branch, Level, Observation},
};
use rgsl::{MatrixF64, VectorF64, blas};

fn lower_triangular_multiply(A: &MatrixF64, B: &mut MatrixF64) {
    blas::level3::dtrmm(
        rgsl::CblasSide::Left,
        rgsl::CblasUplo::Lower,
        rgsl::CblasTranspose::NoTranspose,
        rgsl::CblasDiag::NonUnit,
        1.0,
        A,
        B,
    )
    .unwrap();
}

fn lower_triangular_vector_multiply(A: &MatrixF64, x: &mut VectorF64) {
    blas::level2::dtrmv(
        rgsl::CblasUplo::Lower,
        rgsl::CblasTranspose::NoTranspose,
        rgsl::CblasDiag::NonUnit,
        A,
        x,
    )
    .unwrap();
}

fn lower_triangular_vector_multiply_reverse(x: &mut VectorF64, A: &MatrixF64) {
    blas::level2::dtrmv(
        rgsl::CblasUplo::Lower,
        rgsl::CblasTranspose::Transpose,
        rgsl::CblasDiag::NonUnit,
        A,
        x,
    )
    .unwrap();
}

fn matrix_multiply(A: &MatrixF64, B: &MatrixF64, C: &mut MatrixF64) {
    blas::level3::dgemm(
        rgsl::CblasTranspose::NoTranspose,
        rgsl::CblasTranspose::NoTranspose,
        1.0,
        A,
        B,
        0.0,
        C,
    )
    .unwrap()
}

fn make_square_matrix(n: usize, name: &str) -> MatrixF64 {
    MatrixF64::new(n, n).expect(format!("Failed to allocate {name}!").as_str())
}

pub fn make_x_and_f_matrix(branchs: &[Branch], levels: &[Level]) -> (MatrixF64, VectorF64) {
    // First we construct the x matrix, Eq.2 from Semkow
    let n_levels = levels.len();
    let mut x = MatrixF64::new(n_levels, n_levels).expect("Failed to allocate x matrix.");
    let mut f = VectorF64::new(n_levels).expect("Failed to allocate f vector.");
    for branch in branchs.iter() {
        // This convention follows Semkow
        let j = branch.from;
        let i = branch.to;
        x.set(j, i, branch.val);
    }

    for level in levels.iter() {
        f.set(level.idx, level.feeding);
    }

    // Now we need to ensure that these values sum to 1.
    let norm_f: f64 = f.as_slice().unwrap().iter().sum();
    f.scale(1.0 / norm_f).unwrap();
    // Go through all of the transitions for a state.
    for i in 0..n_levels {
        let mut row = x.get_row(i).unwrap();
        let mut norm: f64 = row.as_slice().unwrap().iter().sum();
        if norm == 0.0 {
            norm = 1.0;
        };
        row.scale(1.0 / norm).unwrap();
        x.set_row(i, &row).unwrap();
    }
    (x, f)
}

pub fn make_tranition_energies(branchs: &[Branch], levels: &[Level]) -> MatrixF64 {
    let n_levels = levels.len();
    let mut energy_matrix =
        MatrixF64::new(n_levels, n_levels).expect("Failed to allocate matrix for gamma energies");
    for branch in branchs.iter() {
        let j = branch.from;
        let i = branch.to;
        energy_matrix.set(j, i, levels[j].energy - levels[i].energy);
    }
    energy_matrix
}

pub fn make_eff_matrix(
    energy_matrix: &MatrixF64,
    peak_spline: &mut Efficiency,
    total_spline: &mut Efficiency,
) -> (MatrixF64, MatrixF64) {
    let n_levels = energy_matrix.size1();
    let mut peak_matrix = MatrixF64::new(n_levels, n_levels)
        .expect("Failed to allocate matrix for peak efficiencies");
    let mut tot_matrix = MatrixF64::new(n_levels, n_levels)
        .expect("Failed to allocate matrix for total efficiencies");
    for j in 0..n_levels {
        for i in 0..n_levels {
            let e = energy_matrix.get(j, i);
            if e > 0.0 {
                peak_matrix.set(j, i, peak_spline.eval(e));
                tot_matrix.set(j, i, total_spline.eval(e));
            }
        }
    }
    (peak_matrix, tot_matrix)
}

#[allow(non_snake_case)]
pub fn calculate_correction(
    x: &MatrixF64,
    f: &VectorF64,
    peak_matrix: &MatrixF64,
    tot_matrix: &MatrixF64,
) -> MatrixF64 {
    let n_levels = f.len();
    // right now there are no internal conversion corrections so c from 4a = x.
    // All of the matrices from Eq.4 of Semkow.
    let mut a = make_square_matrix(n_levels, "a");
    let mut e = make_square_matrix(n_levels, "e");
    let mut b = make_square_matrix(n_levels, "b");
    a.copy_from(x).unwrap();
    a.mul_elements(peak_matrix).unwrap();

    e.copy_from(x).unwrap();
    e.mul_elements(tot_matrix).unwrap();

    b.copy_from(x).unwrap();

    b.sub(&e).unwrap();

    // Now we need the matrices of Eq.5
    let mut A = make_square_matrix(n_levels, "A");
    let mut placeholder = make_square_matrix(n_levels, "temp for A");
    A.copy_from(&a).unwrap();
    placeholder.copy_from(&a).unwrap();
    for i in 1..n_levels {
        lower_triangular_multiply(&a, &mut placeholder);
        A.add(&placeholder).unwrap();
    }

    let mut E = make_square_matrix(n_levels, "E");
    E.set_identity();
    let mut B = make_square_matrix(n_levels, "B");
    B.copy_from(&b).unwrap();

    let mut placeholder = make_square_matrix(n_levels, "temp for B");
    placeholder.copy_from(&b).unwrap();
    for i in 1..n_levels {
        lower_triangular_multiply(&b, &mut placeholder);
        B.add(&placeholder).unwrap();
    }

    B.add(&E).unwrap();

    // N & M from Eq. 6
    let mut N = make_square_matrix(n_levels, "N");
    let mut M = make_square_matrix(n_levels, "M");

    let mut placeholder = VectorF64::new(n_levels).unwrap();
    placeholder.copy_from(f).unwrap();
    lower_triangular_vector_multiply_reverse(&mut placeholder, &B);
    for i in 0..n_levels {
        N.set(i, i, placeholder.get(i));
    }

    // M is simple
    for i in 0..n_levels {
        M.set(i, i, B.get(i, 0));
    }

    // Now we do the no summing correction calculation. Eq.8
    let mut A0 = make_square_matrix(n_levels, "A0");
    let mut B0 = make_square_matrix(n_levels, "B0");
    let mut M0 = make_square_matrix(n_levels, "M0");
    let mut N0 = make_square_matrix(n_levels, "N0");

    A0.copy_from(&a).unwrap();
    M0.copy_from(&E).unwrap();

    // B0 calculation
    let mut placeholder = make_square_matrix(n_levels, "temp for B0");
    B0.copy_from(x).unwrap();
    placeholder.copy_from(&a).unwrap();
    for i in 1..n_levels {
        lower_triangular_multiply(&a, &mut placeholder);
        B0.add(&placeholder).unwrap();
    }

    B0.add(&E).unwrap();

    // N0 calculation
    let mut placeholder = VectorF64::new(n_levels).unwrap();
    placeholder.copy_from(f).unwrap();
    lower_triangular_vector_multiply_reverse(&mut placeholder, &B0);
    for i in 0..n_levels {
        N0.set(i, i, placeholder.get(i));
    }

    // Now we calculate S, which is the sum correction and S0 which is
    // the no sum corrected response. These can then be divided for the correction
    // matrix
    let mut S = make_square_matrix(n_levels, "S");
    let mut S0 = make_square_matrix(n_levels, "S0");
    let mut placeholder = make_square_matrix(n_levels, "temp for S and S0");

    matrix_multiply(&N, &A, &mut placeholder);
    matrix_multiply(&placeholder, &M, &mut S);

    placeholder.set_zero();
    matrix_multiply(&N0, &A0, &mut placeholder);
    matrix_multiply(&placeholder, &M0, &mut S0);

    S0.div_elements(&S).unwrap();

    //    tot_matrix.clone().unwrap()
    S0
}

pub fn correct(obs: Observation, correction: &MatrixF64) -> f64 {
    let j = obs.from;
    let i = obs.to;
    let c = correction.get(j, i);
    c * obs.counts
}
