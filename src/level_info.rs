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
#[derive(Debug)]
pub struct Observation {
    pub from: usize,
    pub to: usize,
    pub counts: f64,
    pub dcounts: f64,
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
}

impl Observation {
    pub fn new(from: usize, to: usize, counts: f64, dcounts: f64) -> Self {
        Self {
            from,
            to,
            counts,
            dcounts,
        }
    }
}
