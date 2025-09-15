// TODO:
// 1) For each length of board in 'supplies' find all possible cuts.
//      For example, if supply len = 10, and needed parts are 3, 5: Then possible cuts are:
//      [3, 3, 3], [5, 3], [5, 5]. Note: Order doesn't matter([5, 3] = [3, 5]).
// 2) Assign values x1, x2 .. xn to the number of boards that would needed to be cut using
//      the specific cut strategy.
// 3) Represent as a linear/integer problem and solve.
// 4) Repeat this process for each unique material type.

use fraction::{Decimal, Zero};

use async_channel::Sender;

use crate::modeling::{CutList, Part, Supply};
use crate::solvers::{Solution, Solver};

pub struct OneDSolver {}

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
