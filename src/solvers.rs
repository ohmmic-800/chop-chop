pub mod naive_solver;

use async_channel::Sender;
use fraction::Decimal;

use crate::modeling::{CutList, Part, Supply};

pub trait Solver {
    fn solve(
        &self,
        supplies: &Vec<Supply>,
        parts: &Vec<Part>,
        progress_sender: Option<Sender<f64>>,
        result_sender: Option<Sender<Result<Solution, String>>>,
    ) -> Result<Solution, String>;

    // TODO: Make this a method?
    fn send_final_result(
        result: Result<Solution, String>,
        progress_sender: Option<Sender<f64>>,
        result_sender: Option<Sender<Result<Solution, String>>>,
    ) {
        if progress_sender.is_some() {
            progress_sender
                .unwrap()
                .send_blocking(1.0)
                .expect("Channel closed");
        }
        if result_sender.is_some() {
            result_sender
                .unwrap()
                .send_blocking(result)
                .expect("Channel closed");
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO: Temporary
pub struct Solution {
    cut_lists: Vec<CutList>,
    supply_consumption: Vec<i64>,
    total_price: Decimal,
}
