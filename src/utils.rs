use fraction::{Decimal, Zero};
use std::collections::HashMap;

use crate::modeling::{Material, Solution};

/// Panics if the keys in `supplies` and `solution` don't match
pub fn compute_supply_consumption(solution: &Solution) -> HashMap<Material, Vec<usize>> {
    let mut consumption = HashMap::new();
    for (material, sub_solution) in solution {
        let mut sub_consumption = vec![0; sub_solution.supplies.len()];
        for cut_list in sub_solution.cut_lists.iter() {
            sub_consumption[cut_list.supply_index] += cut_list.quantity;
        }
        consumption.insert(material.clone(), sub_consumption);
    }
    consumption
}

/// Panics if the keys in `supplies` and `solution` don't match
pub fn compute_total_price(solution: &Solution) -> Decimal {
    let mut total_price = Decimal::zero();
    for sub_solution in solution.values() {
        for cut_list in sub_solution.cut_lists.iter() {
            let unit_price = sub_solution.supplies[cut_list.supply_index].price;
            total_price += unit_price * cut_list.quantity;
        }
    }
    total_price
}
