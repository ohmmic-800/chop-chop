use fraction::{Decimal, Fraction};
use std::collections::HashMap;
use std::hash::Hash;

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
    ) {
        //-> Result<Solution, String> {
        // TODO: Break up supplies and parts by material type.
        let mut supplies_by_material: HashMap<String, Vec<(Fraction, Decimal, i64)>> =
            HashMap::new();
        let mut parts_by_material: HashMap<String, Vec<Part>> = HashMap::new();

        for supply in supplies {
            let material = &supply.material;
            if (!supplies_by_material.contains_key(material)) {
                supplies_by_material.insert(
                    material.clone(),
                    vec![(
                        supply.length.clone(),
                        supply.price.clone(),
                        supply.max_quantity.clone(),
                    )],
                );
            }
        }
        // TODO: Follow process for each material type. Assume 'list' of supplies and parts separated by material.

        // TODO: Make dictionary of cut length needed(for 'this' material). {length, quantity}

        // TODO: Make set of all possible cut combos(That fit within supplies lengths) using needed lengths.

        // TODO: Define constraints and plug into linear solver.

        // TODO: Convert output into cut list.
    }
}
