pub mod naive_solver;

use std::collections::HashMap;

use async_channel::Sender;

use crate::modeling::{CutList, Problem, Solution, SubProblem, SubSolution};

/// Messages that the solver thread can send to the main (UI) thread
pub enum Message {
    Progress(f64),
    SubProgress(f64),
    Results(Result<Solution, String>),
}

pub trait Solver {
    /// Groups cut lists with matching `supply_index` and `part_indices`
    ///
    /// Consumes the old `SubSolution` to create the returned one
    fn group_cut_lists(&self, sub_solution: SubSolution) -> SubSolution {
        let mut counts = HashMap::<(usize, Vec<usize>), usize>::new();
        for cut_list in sub_solution.cut_lists.into_iter() {
            let key = (cut_list.supply_index, cut_list.part_indices);
            let count = counts.get(&key).unwrap_or(&0);
            counts.insert(key, count + cut_list.quantity);
        }
        let mut cut_lists = Vec::new();
        for ((supply_index, part_indices), quantity) in counts.into_iter() {
            cut_lists.push(CutList {
                supply_index,
                part_indices,
                quantity,
            })
        }
        SubSolution {
            cut_lists,
            supplies: sub_solution.supplies,
            parts: sub_solution.parts,
        }
    }

    fn solve(&self, problem: Problem, sender: Option<Sender<Message>>) -> Result<Solution, String> {
        let mut solution = Solution::new();
        for (material, sub_problem) in problem.into_iter() {
            match self.solve_sub_problem(sub_problem, &sender) {
                Ok(sub_solution) => {
                    solution.insert(material, self.group_cut_lists(sub_solution));
                }
                Err(message) => {
                    let message = format!("Error for material \"{}\": {}", material.name, message);
                    let result = Err(message);
                    self.send_result(&sender, result.clone());
                    return result;
                }
            };
        }
        let result = Ok(solution);
        self.send_progress(&sender, 1.0);
        self.send_result(&sender, result.clone());
        result
    }

    fn send_message(&self, sender: &Option<Sender<Message>>, message: Message) {
        if sender.is_some() {
            sender
                .as_ref()
                .unwrap()
                .send_blocking(message)
                .expect("Channel closed");
        }
    }

    fn send_progress(&self, sender: &Option<Sender<Message>>, progress: f64) {
        self.send_message(sender, Message::Progress(progress));
    }

    fn send_result(&self, sender: &Option<Sender<Message>>, result: Result<Solution, String>) {
        self.send_message(sender, Message::Results(result));
    }

    fn send_sub_progress(&self, sender: &Option<Sender<Message>>, sub_progress: f64) {
        self.send_message(sender, Message::SubProgress(sub_progress));
    }

    /// Should consume the `SubProblem` to create the `SubSolution`
    fn solve_sub_problem(
        &self,
        subproblem: SubProblem,
        progress_sender: &Option<Sender<Message>>,
    ) -> Result<SubSolution, String>;
}
