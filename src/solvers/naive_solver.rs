use crate::modeling::{CutList, Part, Supply};
use crate::solvers::{Solution, Solver};

pub struct NaiveSolver {}

impl Solver for NaiveSolver {
    fn solve(&self, supplies: &[&Supply], parts: &[&Part]) -> Result<Solution, String> {
        let mut cut_lists = Vec::<CutList>::new();
        let mut supply_consumption = Vec::<u32>::new();
        let mut total_price = 0.0;
        let mut partial_lengths = Vec::new();
        for _ in supplies {
            supply_consumption.push(0);
        }

        for part in parts {
            for _ in 0..part.quantity {
                let mut done = false;

                // Prioritize cutting from objects already in the cut list
                for (k, cut_item) in cut_lists.iter_mut().enumerate() {
                    if (part.length <= partial_lengths[k])
                        && (part.substance == cut_item.material.substance)
                    {
                        cut_item.cuts.push(part.length);
                        partial_lengths[k] -= part.length;
                        done = true;
                        break;
                    }
                }

                // Then pull from the cheapest supply with large-enough items
                if !done {
                    let mut best_supply = 0;
                    let mut best_price = -1.0;
                    for (i, supply) in supplies.iter().enumerate() {
                        if (part.length <= supply.material.length)
                            && (part.substance == supply.material.substance)
                            && ((supply_consumption[i] < supply.max_quantity)
                                || (supply.max_quantity == 0))
                            && ((best_price < 0.0) || (supply.price < best_price))
                        {
                            best_supply = i;
                            best_price = supply.price;
                        }
                    }
                    if best_price < 0.0 {
                        // May be triggered even if valid solutions exist
                        return Err(String::from("No materials available with sufficient size"));
                    } else {
                        // TODO: Is .clone() the right way to do this?
                        cut_lists.push(CutList {
                            material: supplies[best_supply].material.clone(),
                            cuts: vec![part.length],
                        });
                        supply_consumption[best_supply] += 1;
                        total_price += supplies[best_supply].price;
                        partial_lengths.push(supplies[best_supply].material.length - part.length);
                    }
                }
            }
        }
        Ok(Solution {
            cut_lists,
            supply_consumption,
            total_price,
        })
    }
}

// https://doc.rust-lang.org/stable/book/ch11-01-writing-tests.html
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modeling::Material;

    #[test]
    fn test_naive_solver() {
        // TODO: Is .clone() the right way to do this?
        let substance = String::from("Pine 2x4");
        let material = Material {
            substance: substance.clone(),
            length: 8.0,
        };
        let on_hand_supply = Supply {
            material: material.clone(),
            price: 0.0,
            max_quantity: 1,
        };
        let purchaseable_supply = Supply {
            material: material,
            price: 3.50,
            max_quantity: 0,
        };
        let part_1 = Part {
            substance: substance.clone(),
            length: 3.0,
            quantity: 3,
        };
        let part_2 = Part {
            substance: substance,
            length: 1.5,
            quantity: 1,
        };
        let solution = NaiveSolver {}
            .solve(
                &[&on_hand_supply, &purchaseable_supply],
                &[&part_1, &part_2],
            )
            .unwrap();
        assert_eq!(solution.supply_consumption, vec![1, 1]);
        assert_eq!(solution.total_price, 3.5);
    }
}
