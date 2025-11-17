use std::collections::HashMap;
use std::str::FromStr;

use adw::prelude::*;
use fraction::{Decimal, Fraction, Zero};

use super::entry::EntryData;
use crate::modeling::{
    Dimension, Material, Part, Problem, Solution, SubProblem, SubSolution, Supply,
};
use crate::size::{FractionFormat, Size, SizeUnit};

// Put `from` methods here because selection indices are UI-specific

impl Dimension {
    pub fn from(selection: u32) -> Self {
        match selection {
            0 => Self::OneD,
            1 => Self::TwoD,
            _ => panic!(),
        }
    }
}

impl FractionFormat {
    pub fn from(selection: u32, precision: u32) -> Self {
        match selection {
            0 => Self::Decimal(precision as usize),
            1 => Self::Mixed,
            2 => Self::Fraction,
            _ => panic!(),
        }
    }
}

impl Size {
    pub fn from(selection: u32, major: &str, minor: &str) -> Self {
        Self {
            unit: SizeUnit::from(selection),
            major: parse_positive_fraction(major, true).unwrap(),
            minor: parse_positive_fraction(minor, true).unwrap(),
        }
    }
}

impl SizeUnit {
    pub fn from(selection: u32) -> Self {
        match selection {
            0 => Self::FeetInches,
            1 => Self::Inches,
            2 => Self::Meters,
            3 => Self::Centimeters,
            _ => panic!(),
        }
    }
}

/// For Option<Result<Solution, String>>> serialization (required due to HashMap)
pub fn flatten_results(
    results: Option<Result<Solution, String>>,
) -> Option<Result<Vec<(Material, SubSolution)>, String>> {
    match results {
        Some(Ok(hashmap)) => Some(Ok(hashmap.into_iter().collect())),
        Some(Err(message)) => Some(Err(message)),
        None => None,
    }
}

pub fn format_price(price: fraction::Decimal, precision: u32) -> String {
    if price.is_zero() {
        String::from("Free")
    } else {
        // Conversion to f64 is required for correct rounding
        let value: f64 = price.try_into().unwrap();

        format!("${0:.1$}", value, precision as usize)
    }
}

pub fn format_quantity(quantity: i64) -> String {
    if quantity == -1 {
        String::from("Unlimited")
    } else {
        quantity.to_string()
    }
}

pub fn generate_problem(
    supply_entry_data: Vec<EntryData>,
    part_entry_data: Vec<EntryData>,
    blade_width: Size,
) -> Problem {
    let mut problem = Problem::new();

    for entry_data in supply_entry_data {
        let material = Material {
            name: entry_data.material.clone(),
            dimension: Dimension::from(entry_data.dimension),
        };
        let supply = Supply {
            name: entry_data.name.clone(),
            length: parse_length(&entry_data),
            price: parse_price(&entry_data.price, true).unwrap(),
            max_quantity: parse_quantity(&entry_data.quantity, true).unwrap(),
        };
        match problem.get_mut(&material) {
            Some(sub_problem) => {
                sub_problem.supplies.push(supply);
            }
            None => {
                let sub_problem = SubProblem {
                    supplies: vec![supply],
                    parts: vec![],
                    blade_width: blade_width.clone(),
                };
                problem.insert(material, sub_problem);
            }
        }
    }

    for entry_data in part_entry_data {
        let material = Material {
            name: entry_data.material.clone(),
            dimension: Dimension::from(entry_data.dimension),
        };
        let part = Part {
            name: entry_data.name.clone(),
            length: parse_length(&entry_data),
            quantity: parse_quantity(&entry_data.quantity, true).unwrap(),
        };
        match problem.get_mut(&material) {
            Some(sub_problem) => {
                sub_problem.parts.push(part);
            }
            None => {
                let sub_problem = SubProblem {
                    supplies: vec![],
                    parts: vec![part],
                    blade_width: blade_width.clone(),
                };
                problem.insert(material, sub_problem);
            }
        }
    }

    problem
}

pub fn parse_positive_fraction(text: &str, allow_empty: bool) -> Result<Fraction, ()> {
    let tokens: Vec<_> = text.trim().split(" ").filter(|s| !s.is_empty()).collect();
    if (tokens.is_empty() && !allow_empty) || (tokens.len() > 2) {
        Err(())
    } else {
        let mut size = Fraction::zero();
        for token in tokens {
            size += match Fraction::from_str(token) {
                Ok(value) if (!value.is_sign_negative()) => value,
                _ => return Err(()),
            };
        }
        Ok(size)
    }
}

// Currently allows prices as fractions
pub fn parse_price(text: &str, allow_empty: bool) -> Result<Decimal, ()> {
    let text = text.trim();
    if text.is_empty() && !allow_empty {
        Err(())
    } else if text.is_empty() {
        Ok(Decimal::zero())
    } else {
        match Decimal::from_str(text) {
            Ok(value) if (value >= Decimal::zero()) => Ok(value),
            _ => Err(()),
        }
    }
}

pub fn parse_quantity(text: &str, allow_empty: bool) -> Result<i64, ()> {
    let text = text.trim();
    if text.is_empty() && !allow_empty {
        Err(())
    } else if text.is_empty() {
        Ok(-1)
    } else {
        match text.parse::<i64>() {
            Ok(value) if (value >= 0) => Ok(value),
            _ => Err(()),
        }
    }
}

/// For Option<Result<Solution, String>>> deserialization (required due to HashMap)
pub fn unflatten_results(
    results: Option<Result<Vec<(Material, SubSolution)>, String>>,
) -> Option<Result<Solution, String>> {
    match results {
        Some(Ok(vec)) => {
            let mut hashmap = HashMap::new();
            for (key, val) in vec.into_iter() {
                hashmap.insert(key, val);
            }
            Some(Ok(hashmap))
        }
        Some(Err(message)) => Some(Err(message)),
        None => None,
    }
}

pub fn validate_entry<F>(entry: &adw::EntryRow, reference: Option<String>, validate: F) -> bool
where
    F: Fn(&adw::EntryRow) -> bool,
{
    let is_valid = validate(entry);
    if !is_valid {
        entry.add_css_class("invalid-entry");
    } else {
        entry.remove_css_class("invalid-entry");
    }
    if is_valid
        && let Some(s) = reference
        && entry.text() != s
    {
        entry.set_show_apply_button(true);
    } else {
        // Toggle the apply button so it will activate on the next change
        entry.set_show_apply_button(false);
        entry.set_show_apply_button(true);
    }
    is_valid
}

fn parse_length(entry_data: &EntryData) -> Size {
    Size {
        unit: SizeUnit::from(entry_data.length_unit),
        major: parse_positive_fraction(&entry_data.major_length, true).unwrap(),
        minor: parse_positive_fraction(&entry_data.minor_length, true).unwrap(),
    }
}
