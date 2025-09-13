pub mod naive_solver;

use fraction::Decimal;

use crate::modeling::{CutList, Part, Supply};

pub trait Solver {
    fn solve(&self, supplies: &Vec<Supply>, parts: &Vec<Part>) -> Result<Solution, String>;
}

#[derive(Debug)]
#[allow(dead_code)] // TODO: Temporary
pub struct Solution {
    cut_lists: Vec<CutList>,
    supply_consumption: Vec<i64>,
    total_price: Decimal,
}
