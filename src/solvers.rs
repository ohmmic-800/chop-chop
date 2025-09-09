pub mod naive_solver;

use crate::modeling::{CutList, Part, Supply};

pub trait Solver {
    fn solve(&self, supplies: &[&Supply], parts: &[&Part]) -> Result<Solution, String>;
}

#[derive(Debug)]
#[allow(dead_code)] // TODO: Temporary
pub struct Solution {
    cut_lists: Vec<CutList>,
    supply_consumption: Vec<u32>,
    total_price: f64,
}
