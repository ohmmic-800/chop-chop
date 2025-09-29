use fraction::ToPrimitive;
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
        let m = parts.len();
        let n = supplies.len();

        let mut vars = variables!();

        // Binary usage variables
        let u: Vec<_> = (0..n).map(|_| vars.add(variable().binary())).collect();

        // Assignment variables x[i][j]
        let mut x: Vec<Vec<_>> = Vec::with_capacity(n);
        for _ in 0..n {
            let mut row = Vec::with_capacity(m);
            for _ in 0..m {
                row.push(vars.add(variable().binary()));
            }
            x.push(row);
        }

        // Build objective: minimize sum(price_i * u_i)
        let mut obj_expr = Expression::from_other_affine(0.0);
        for i in 0..n {
            let price_f64 = supplies[i]
                .price
                .to_f64()
                .ok_or("Failed to convert Decimal price to f64")?;
            obj_expr = obj_expr + u[i] * price_f64;
        }

        let mut model = vars.minimise(obj_expr).using(default_solver);

        // Each part must be assigned exactly once
        for j in 0..m {
            let mut sum_expr = Expression::from_other_affine(0.0);
            for i in 0..n {
                sum_expr = sum_expr + x[i][j];
            }
            model = model.with(constraint!(sum_expr == 1.0)); // must be f64
        }

        // Capacity constraints: sum of lengths assigned to supply i <= supply length
        for i in 0..n {
            let mut cap_expr = Expression::from_other_affine(0.0);
            for j in 0..m {
                let length_f64 = parts[j]
                    .length
                    .to_f64()
                    .ok_or("Failed to convert part length to f64")?;
                cap_expr = cap_expr + length_f64 * x[i][j];
            }
            let supply_len = supplies[i]
                .length
                .to_f64()
                .ok_or("Failed to convert supply length to f64")?;
            model = model.with(constraint!(cap_expr <= supply_len));
        }

        // Linking constraints: if x[i][j] = 1, force u[i] = 1
        for i in 0..n {
            for j in 0..m {
                model = model.with(constraint!(u[i] >= x[i][j]));
            }
        }

        let solution: Solution = model.solve()?;

        println!("Optimal cost = {}", solution.eval(&model.objective()));
        for i in 0..n {
            if solution.value(u[i]) > 0.5 {
                println!("Supply {} is used", i);
                for j in 0..m {
                    if solution.value(x[i][j]) > 0.5 {
                        println!("  Part {} assigned", j);
                    }
                }
            }
        }

        Ok(())
    }
}
