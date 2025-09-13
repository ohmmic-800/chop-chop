use fraction::{Decimal, Zero};

use async_channel::Sender;

use crate::modeling::{CutList, Part, Supply};
use crate::solvers::{Solution, Solver};

pub struct NaiveSolver {}

impl Solver for NaiveSolver {
    fn solve(
        &self,
        supplies: &Vec<Supply>,
        parts: &Vec<Part>,
        progress_sender: Option<Sender<f64>>,
        result_sender: Option<Sender<Result<Solution, String>>>,
    ) -> Result<Solution, String> {
        let mut cut_lists = Vec::<CutList>::new();
        let mut supply_consumption = Vec::<i64>::new();
        let mut total_price = Decimal::zero();
        let mut partial_lengths = Vec::new();
        for _ in supplies {
            supply_consumption.push(0);
        }

        for part in parts {
            for _ in 0..part.quantity {
                let mut done = false;

                // Prioritize cutting from objects already in the cut list
                for (k, cut_item) in cut_lists.iter_mut().enumerate() {
                    if (part.length <= partial_lengths[k]) && (part.material == cut_item.material) {
                        cut_item.cuts.push(part.length);
                        partial_lengths[k] -= part.length;
                        done = true;
                        break;
                    }
                }

                // Then pull from the cheapest supply with large-enough items
                if !done {
                    let mut best_supply = 0;
                    let mut best_price = Decimal::infinity();
                    for (i, supply) in supplies.iter().enumerate() {
                        if (part.length <= supply.length)
                            && (part.material == supply.material)
                            && ((supply_consumption[i] < supply.max_quantity)
                                || (supply.max_quantity == -1))
                            && (supply.price < best_price)
                        {
                            best_supply = i;
                            best_price = supply.price;
                        }
                    }
                    if best_price == Decimal::infinity() {
                        // May be triggered even if valid solutions exist
                        let result =
                            Err(String::from("No materials available with sufficient size"));
                        Self::send_final_result(result.clone(), progress_sender, result_sender);
                        return result;
                    } else {
                        // TODO: Is .clone() the right way to do this?
                        cut_lists.push(CutList {
                            material: supplies[best_supply].material.clone(),
                            length: supplies[best_supply].length,
                            cuts: vec![part.length],
                        });
                        supply_consumption[best_supply] += 1;
                        total_price += supplies[best_supply].price;
                        partial_lengths.push(supplies[best_supply].length - part.length);
                    }
                }
            }
        }
        let solution = Ok(Solution {
            cut_lists,
            supply_consumption,
            total_price,
        });
        Self::send_final_result(solution.clone(), progress_sender, result_sender);
        solution
    }
}

// https://doc.rust-lang.org/stable/book/ch11-01-writing-tests.html
#[cfg(test)]
mod tests {
    use super::*;

    use fraction::Fraction;

    #[test]
    fn test_naive_solver() {
        // TODO: Is .clone() the right way to do this?
        let material = String::from("Pine 2x4");
        let on_hand_supply = Supply {
            material: material.clone(),
            length: Fraction::from(8.0),
            price: Decimal::zero(),
            max_quantity: 1,
        };
        let purchaseable_supply = Supply {
            material: material.clone(),
            length: Fraction::from(8.0),
            price: Decimal::from(3.5),
            max_quantity: 0,
        };
        let part_1 = Part {
            material: material.clone(),
            length: Fraction::from(3.0),
            quantity: 3,
        };
        let part_2 = Part {
            material,
            length: Fraction::from(1.5),
            quantity: 1,
        };
        let solution = NaiveSolver {}
            .solve(
                &vec![on_hand_supply, purchaseable_supply],
                &vec![part_1, part_2],
                None,
                None,
            )
            .unwrap();
        assert_eq!(solution.supply_consumption, vec![1, 1]);
        assert_eq!(solution.total_price, Decimal::from(3.5));
    }
}
