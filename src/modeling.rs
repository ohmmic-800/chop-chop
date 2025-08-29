/// A substance available at a specific dimension
///
/// Assumed to be 1D for now
///
/// Examples:
/// - 8-foot pine 2x4
/// - 100-meter spoon of green yarn
#[derive(Clone, Debug)]
pub struct Material {
    pub substance: String,
    pub length: f32,
}

/// Represents an available supply of a material
///
/// A price of zero indicates that the material is on-hand (free)
///
/// A max_quantity of zero indicates that the supply is unlimited
///
/// Examples:
/// - There are five 8-foot pine 2x4s on-hand (price $0, max_quantity 5)
/// - 100-meter spools of yarn are purchasable for $100 each (price $100, max_quantity 0)
#[derive(Debug)]
pub struct Supply {
    pub material: Material,
    pub price: f32,
    pub max_quantity: u32,
}

/// Represents a desired part
///
/// Assumed 1D for now
#[derive(Debug)]
pub struct Part {
    pub substance: String,
    pub length: f32,
    pub quantity: u32,
}

/// Represents a set of of cuts to perform on an object (instance of a material)
///
/// Assumed 1D for now
#[derive(Debug)]
pub struct CutList {
    pub material: Material,
    pub cuts: Vec<f32>,
}
