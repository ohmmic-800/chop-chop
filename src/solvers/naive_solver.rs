use async_channel::Sender;
use fraction::Decimal;

use crate::modeling::{CutList, SubProblem, SubSolution};
use crate::solvers::{Message, Solver};

pub struct NaiveSolver {}

impl Solver for NaiveSolver {
    fn solve_sub_problem(
        &self,
        sub_problem: SubProblem,
        sender: &Option<Sender<Message>>,
    ) -> Result<SubSolution, String> {
        let SubProblem {
            parts,
            supplies,
            blade_width,
        } = sub_problem;
        let mut cut_lists = Vec::<CutList>::new();
        let mut supply_consumption = vec![0; supplies.len()];
        let mut partial_lengths = Vec::new();

        let mut progress = 0.0;
        let total_count = parts.iter().map(|p| p.quantity).sum::<i64>();

        for (i, part) in parts.iter().enumerate() {
            let part_meters = part.length.to_meters();
            for _ in 0..part.quantity {
                let mut done = false;

                // Prioritize cutting from objects already in the cut list
                for (j, cut_item) in cut_lists.iter_mut().enumerate() {
                    if part.length.to_meters() <= partial_lengths[j] {
                        cut_item.part_indices.push(i);
                        partial_lengths[j] -= part_meters + blade_width.to_meters();
                        done = true;
                        break;
                    }
                }

                // Then pull from the cheapest supply with large-enough items
                if !done {
                    let mut best_supply = 0;
                    let mut best_price = Decimal::infinity();
                    for (i, supply) in supplies.iter().enumerate() {
                        if (part_meters <= supply.length.to_meters())
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
                        return Err(String::from("No materials available with sufficient size"));
                    } else {
                        cut_lists.push(CutList {
                            supply_index: best_supply,
                            part_indices: vec![i],
                            quantity: 1,
                        });
                        supply_consumption[best_supply] += 1;
                        partial_lengths.push(
                            supplies[best_supply].length.to_meters()
                                - part_meters
                                - blade_width.to_meters(),
                        );
                    }
                }

                progress += 1.0 / (total_count as f64);
                self.send_sub_progress(sender, progress);
            }
        }

        Ok(SubSolution {
            cut_lists,
            supplies,
            parts,
        })
    }
}

// https://doc.rust-lang.org/stable/book/ch11-01-writing-tests.html
#[cfg(test)]
mod tests {
    use fraction::Zero;

    use super::*;
    use crate::modeling::{Dimension, Material, Part, Problem, Supply};
    use crate::size::Size;
    use crate::utils::compute_total_price;

    #[test]
    fn test_naive_solver() {
        let material = Material {
            name: String::from("Pine 2x4"),
            dimension: Dimension::OneD,
        };
        let supplies = vec![
            Supply {
                name: String::new(),
                length: Size::from_meters(8.0),
                price: Decimal::zero(),
                max_quantity: 1,
            },
            Supply {
                name: String::new(),
                length: Size::from_meters(8.0),
                price: Decimal::from(3.5),
                max_quantity: -1,
            },
        ];
        let parts = vec![
            Part {
                name: String::new(),
                length: Size::from_meters(3.0),
                quantity: 3,
            },
            Part {
                name: String::new(),
                length: Size::from_meters(1.5),
                quantity: 1,
            },
        ];
        let blade_width = Size::from_meters(0.0);
        let mut problem = Problem::new();
        problem.insert(
            material.clone(),
            SubProblem {
                supplies,
                parts,
                blade_width,
            },
        );
        let solution = NaiveSolver {}.solve(problem, None).unwrap();
        assert_eq!(compute_total_price(&solution), Decimal::from(3.5));
    }
}
