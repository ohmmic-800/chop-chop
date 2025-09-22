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
            // TODO:
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
