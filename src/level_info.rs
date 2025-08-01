use rand::prelude::*;
use rand_distr::{Distribution, Normal};
use rgsl::MatrixF64;
use statistical::{mean, standard_deviation};

#[derive(Debug, Clone)]
pub struct Level {
    pub idx: usize,
    pub energy: f64,
    pub denergy: f64,
    pub feeding: f64,
    pub dfeeding: f64,
}
#[derive(Debug)]
pub struct Branch {
    pub from: usize,
    pub to: usize,
    pub val: f64,
    pub dval: f64,
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub from: usize,
    pub to: usize,
    pub counts: f64,
    pub dcounts: f64,
    pub correction_samples: Vec<f64>,
}

fn truncated_normal(mu: f64, std: f64, r: &mut ThreadRng) -> f64 {
    if mu == 0.0 {
        return 0.0;
    }
    let norm = Normal::new(mu, std).unwrap();
    loop {
        let s: f64 = norm.sample(r);
        if s > 0.0 {
            break s;
        }
    }
}

impl Level {
    pub fn new(idx: usize, energy: f64, denergy: f64, feeding: f64, dfeeding: f64) -> Self {
        Self {
            idx,
            energy,
            denergy,
            feeding,
            dfeeding,
        }
    }

    pub fn sample(&self, r: &mut ThreadRng) -> Self {
        let idx = self.idx;
        let energy = self.energy;
        let denergy = self.denergy;
        let feeding = truncated_normal(self.feeding, self.dfeeding, r);
        let dfeeding = 0.0;

        Self {
            idx,
            energy,
            denergy,
            feeding,
            dfeeding,
        }
    }
}

impl Branch {
    pub fn new(from: usize, to: usize, val: f64, dval: f64) -> Self {
        Self {
            from,
            to,
            val,
            dval,
        }
    }
    pub fn sample(&self, r: &mut ThreadRng) -> Self {
        let from = self.from;
        let to = self.to;
        let val = truncated_normal(self.val, self.dval, r);
        let dval = 0.0;

        Self {
            from,
            to,
            val,
            dval,
        }
    }
}

impl Observation {
    pub fn new(from: usize, to: usize, counts: f64, dcounts: f64, n_samples: usize) -> Self {
        Self {
            from,
            to,
            counts,
            dcounts,
            correction_samples: vec![0.0; n_samples],
        }
    }

    pub fn add_correction(&mut self, idx: usize, m: &MatrixF64) {
        self.correction_samples[idx] = m.get(self.from, self.to);
    }

    pub fn corrected_value(&mut self) -> Result<(f64, f64), ()> {
        let c = mean(&self.correction_samples);
        if c.is_nan() {
            return Err(());
        }
        let dc = standard_deviation(&self.correction_samples, None);
        let val = self.counts * c;
        let dval = val * f64::sqrt((dc / c).powi(2) + (self.dcounts / self.counts).powi(2));
        Ok((val, dval))
    }
}
