use std::fs;

/// This module handles the creating the splines for efficiency.
use rgsl::interpolation;
use rgsl::{Interp, InterpAccel, InterpType};

pub struct Efficiency {
    pub energies: Vec<f64>,
    pub eff: Vec<f64>,
    interp: Interp,
    interp_acc: InterpAccel,
}

impl Efficiency {
    fn new(energies: Vec<f64>, eff: Vec<f64>) -> Self {
        let interp_type = InterpType::cspline();
        let mut interp =
            Interp::new(interp_type, energies.len()).expect("Failed to initialize cubic spline");
        interp.init(&energies, &eff);
        let mut interp_acc = InterpAccel::new();
        Self {
            energies,
            eff,
            interp,
            interp_acc,
        }
    }

    pub fn eval(&mut self, energy: f64) -> f64 {
        interpolation::eval(
            &self.interp,
            &self.energies,
            &self.eff,
            energy,
            &mut self.interp_acc,
        )
    }
}
pub fn make_efficiency(file_path: &str) -> Efficiency {
    let file_content =
        fs::read_to_string(file_path).expect(format!("Failed to read: {file_path}\n").as_str());

    let mut energies: Vec<f64> = Vec::new();
    let mut eff: Vec<f64> = Vec::new();
    for line in file_content.lines() {
        let nums: Vec<f64> = line
            .split_whitespace()
            .map(|p| p.parse().expect("Failed to parse line in efficiency file!"))
            .collect();
        energies.push(nums[0]);
        eff.push(nums[1]);
    }

    Efficiency::new(energies, eff)
}
