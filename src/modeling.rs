use fraction::{Decimal, Fraction};

/// Represents an available supply of a material
///
/// Examples:
/// - There are five 8-foot pine 2x4s on-hand (price $0, max_quantity 5)
/// - 100-meter spools of yarn are purchasable for $100 each (price $100, max_quantity 0)
#[derive(Debug)]
pub struct Supply {
    pub material: String,
    pub length: Fraction,

    /// Zero indicates that the material is on-hand (free)
    pub price: Decimal,

    /// -1 indicates that the supply is unlimited
    pub max_quantity: i64,
}

impl Clone for Supply {
    fn clone(&self) -> Self {
        Supply {
            material: self.material.clone(),
            length: self.length.clone(), // Fraction must implement Clone
            price: self.price.clone(),
            max_quantity: self.max_quantity,
        }
    }
}

/// Represents a desired part
///
/// Assumed 1D for now
#[derive(Debug)]
pub struct Part {
    pub material: String,
    pub length: Fraction,
    pub quantity: i64,
}

impl Clone for Part {
    fn clone(&self) -> Self {
        Part {
            material: self.material.clone(),
            length: self.length.clone(), // Fraction must implement Clone
            quantity: self.quantity,
        }
    }
}

/// Represents a set of of cuts to perform on an object
///
/// Assumed 1D for now
#[derive(Debug, Clone)]
pub struct CutList {
    pub material: String,

    /// The original length before cutting
    pub length: Fraction,

    pub cuts: Vec<Fraction>,
}
