use fraction::{Decimal, Fraction};
use good_lp::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::AddAssign;

use async_channel::Sender;

use crate::modeling::{CutList, Part, Supply};
use crate::solvers::{Solution, Solver};

pub struct OneDSolver {}
// # TODO: Account for blade length.
impl Solver for OneDSolver {
    fn solve(
        &self,
        supplies: &Vec<Supply>,
        parts: &Vec<Part>,
        progress_sender: Option<Sender<f64>>,
        result_sender: Option<Sender<Result<Solution, String>>>,
    ) -> Result<Solution, String> {
       
    }
}
