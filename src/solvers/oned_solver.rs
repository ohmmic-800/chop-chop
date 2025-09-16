use fraction::{Decimal, Fraction};
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
    ) {
        // -> Result<Solution, String> {

        //Break up supplies and parts by material type.
        let mut supplies_by_material: HashMap<String, Vec<(Fraction, Decimal, i64)>> =
            HashMap::new();
        let mut parts_by_material: HashMap<String, Vec<(Fraction, i64)>> = HashMap::new();

        for supply in supplies {
            let material = &supply.material;
            supplies_by_material
                .entry(material.clone()) // clone the key if needed
                .or_insert_with(Vec::new) // insert empty vector if key doesn't exist
                .push((
                    supply.length.clone(),
                    supply.price.clone(),
                    supply.max_quantity.clone(),
                ));
        }
        for part in parts {
            let material = &part.material;
            parts_by_material
                .entry(material.clone())
                .or_insert_with(Vec::new)
                .push((part.length.clone(), part.quantity.clone()));
        }

        let mut possible_cuts: Vec<Vec<Fraction>> = Vec::new();
        for material in parts_by_material.keys() {
            // Make set of cut length needed(for 'this' material).
            let mut cuts_set: HashSet<Fraction> = HashSet::new();
            if let Some(parts) = parts_by_material.get(material) {
                for part in parts {
                    cuts_set.insert(part.0);
                }
            }
            let mut cuts: Vec<Fraction> = cuts_set.iter().cloned().collect();
            cuts.sort();
            // Make set of supply lengths available.
            let mut pieces_set: HashSet<Fraction> = HashSet::new();
            if let Some(supplies) = supplies_by_material.get(material) {
                for supply in supplies {
                    pieces_set.insert(supply.0);
                }
            }
            let mut pieces: Vec<Fraction> = pieces_set.iter().cloned().collect();
            pieces.sort();
            // Compute all possible cut possibilities for each supply.
            for piece in pieces {
                generate_combinations(&mut cuts, piece, 0, &mut Vec::new(), &mut possible_cuts);
            }
        }

        // TODO: Define constraints and plug into linear solver.

        // TODO: Convert output into cut list.
    }
}

// Generates possible cut combinations for a single piece.
fn generate_combinations(
    cuts: &Vec<Fraction>,
    piece: Fraction,
    start_index: usize,
    current: &mut Vec<Fraction>,
    results: &mut Vec<Vec<Fraction>>,
) {
    let mut sum: Fraction = Fraction::new(0u64, 1u64);
    for cut in current.iter() {
        sum.add_assign(cut.clone());
    }
    if (sum > piece) {
        return;
    }
    results.push(current.clone());

    for i in start_index..cuts.len() {
        let mut clone = current.clone();
        clone.push(cuts.get(i).unwrap().clone());
        generate_combinations(cuts, piece, i, &mut clone, results);
    }
}
