use std::str::FromStr;

use fraction::{Decimal, Fraction, Zero};

use super::entry::EntryData;
use crate::modeling::{Part, Supply};
use crate::units::{LengthUnit, to_meters};

pub fn format_length(length: Fraction, sublength: Fraction, unit: LengthUnit) -> String {
    let mut output = format!("{:.10}{}", length, unit.main_symbol());
    if unit.has_subunit() {
        output += &format!(" {:.10}{}", sublength, unit.subunit_symbol());
    }
    output
}

pub fn format_price(price: fraction::Decimal) -> String {
    if price == Decimal::zero() {
        String::from("Free")
    } else {
        format!("${:.2}", price)
    }
}

pub fn format_quantity(quantity: i64) -> String {
    if quantity == -1 {
        String::from("Unlimited")
    } else {
        quantity.to_string()
    }
}

pub fn parse_length(text: &str, allow_empty: bool) -> Result<Fraction, ()> {
    let tokens: Vec<_> = text.trim().split(" ").filter(|s| !s.is_empty()).collect();
    if (tokens.is_empty() && !allow_empty) || (tokens.len() > 2) {
        Err(())
    } else {
        let mut length = Fraction::zero();
        for token in tokens {
            length += match Fraction::from_str(token) {
                Ok(value) if (value >= Fraction::zero()) => value,
                _ => return Err(()),
            };
        }
        Ok(length)
    }
}

pub fn parse_length_unit(selection: u32) -> LengthUnit {
    match selection {
        0 => LengthUnit::FeetInches,
        1 => LengthUnit::Inches,
        2 => LengthUnit::Meters,
        3 => LengthUnit::Centimeters,
        _ => panic!(),
    }
}

pub fn parse_part_entries(part_entry_data: Vec<EntryData>) -> Vec<Part> {
    let mut parts = Vec::<Part>::with_capacity(part_entry_data.len());
    for entry_data in part_entry_data {
        let length = to_meters(
            parse_length(&entry_data.length, false).unwrap(),
            parse_length(&entry_data.sublength, true).unwrap(),
            parse_length_unit(entry_data.length_unit),
        );
        parts.push(Part {
            material: entry_data.material,
            length: length,
            quantity: parse_quantity(&entry_data.quantity, false).unwrap(),
        });
    }
    parts
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

pub fn parse_supply_entries(supply_entry_data: Vec<EntryData>) -> Vec<Supply> {
    let mut supplies = Vec::<Supply>::with_capacity(supply_entry_data.len());
    for entry_data in supply_entry_data {
        let length = to_meters(
            parse_length(&entry_data.length, false).unwrap(),
            parse_length(&entry_data.sublength, true).unwrap(),
            parse_length_unit(entry_data.length_unit),
        );
        supplies.push(Supply {
            material: entry_data.material,
            length: length,
            price: parse_price(&entry_data.price, true).unwrap(),
            max_quantity: parse_quantity(&entry_data.quantity, true).unwrap(),
        });
    }
    supplies
}
