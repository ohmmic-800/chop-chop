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
        // Break up supplies and parts by material type.
        let mut supplies_by_material: HashMap<String, Vec<Supply>> = HashMap::new();
        let mut parts_by_material: HashMap<String, Vec<Part>> = HashMap::new();

        for supply in supplies {
            let material = &supply.material;
            supplies_by_material
                .entry(material.clone()) // clone the key if needed
                .or_insert_with(Vec::new) // insert empty vector if key doesn't exist
                .push(supply.clone());
        }
        for part in parts {
            let material = &part.material;
            parts_by_material
                .entry(material.clone())
                .or_insert_with(Vec::new)
                .push(part.clone());
        }

        for material in parts_by_material.keys() {
            // Make list of parts needed for the current material.
            let mut cuts_set: Vec<Part> = Vec::new();
            if let Some(parts) = parts_by_material.get(material) {
                for part in parts {
                    cuts_set.push(part.clone());
                }
            }
            // Make list of supplies available for current material.
            let mut supply_set: Vec<Supply> = Vec::new();
            if let Some(supplies) = supplies_by_material.get(material) {
                for supply in supplies {
                    supply_set.push(supply.clone());
                }
            }
            let mut possible_cuts: Vec<(Supply, Vec<Fraction>)> = Vec::new(); // Vec< Thing to cut <How it will be cut>>.
            // Compute all possible cut possibilities for each supply.
            for piece in supply_set {
                generate_combinations(
                    &mut cuts_set,
                    &piece,
                    0,
                    &mut (piece, Vec::new()),
                    &mut possible_cuts,
                );
            }
            // TODO: Define constraints and plug into linear solver.
            // TODO: Define price constraint.
            let solver = default_solver();
            let mut problem = Problem::new(solver).minimise(0.0);

            // Create continuous variables
            let mut vars = Vec::new();
            for _ in &possible_cuts {
                vars.push(problem.add_variable(variable().min(0.0))); // continuous
            }

            // Objective: total cost
            let mut objective = 0.0;
            for (i, (supply, _pattern)) in possible_cuts.iter().enumerate() {
                objective += supply.price * vars[i];
            }
            problem.set_objective(objective);

            // Constraint: each part must be satisfied
            for part in &cuts_set {
                let mut lhs: Expression = 0.0;
                for (i, (_supply, pattern)) in possible_cuts.iter().enumerate() {
                    let count = pattern
                        .iter()
                        .filter(|&&length| length == part.length)
                        .count() as f64;
                    lhs += count * vars[i];
                }
                problem.add_constraint(lhs >= part.quantity as f64);
            }

            // Constraint: do not exceed supply quantity
            for (i, (supply, _pattern)) in possible_cuts.iter().enumerate() {
                problem.add_constraint(vars[i] <= supply.max_quantity);
            }

            // Solve
            let solution = problem.solve().unwrap();
        }

        // TODO: Convert output into cut list.

        // TODO: Change to return actual value.
        let solution = Ok(Solution {
            cut_lists,
            supply_consumption,
            total_price,
        });
        self.send_final_result(solution.clone(), progress_sender, result_sender);
        solution
    }
}

// Generates possible cut combinations for a single piece.
fn generate_combinations(
    cuts: &Vec<Part>,
    piece: &Supply,
    start_index: usize,
    current: &mut (Supply, Vec<Fraction>),
    results: &mut Vec<(Supply, Vec<Fraction>)>,
) {
    // Check if sum of input cuts are longer than provided supply.
    let mut sum: Fraction = Fraction::new(0u64, 1u64);
    for cut in current.1.iter() {
        sum.add_assign(cut.clone());
    }
    if sum > piece.length {
        return;
    }
    results.push(current.clone());

    generate_combinations(cuts, piece, start_index, current, results);
    for i in start_index..cuts.len() {
        let mut clone = current.clone();
        clone.1.push((cuts.get(i).unwrap().length.clone()));
        generate_combinations(cuts, piece, i, &mut clone, results);
    }
}
