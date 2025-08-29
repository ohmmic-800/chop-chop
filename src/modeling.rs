/// Represents an available supply of a material
///
/// Examples:
/// - There are five 8-foot pine 2x4s on-hand (price $0, max_quantity 5)
/// - 100-meter spools of yarn are purchasable for $100 each (price $100, max_quantity 0)
#[derive(Debug)]
pub struct Supply {
    pub material: String,
    pub length: f32,

    /// Zero indicates that the material is on-hand (free)
    pub price: f32,

    /// Zero indicates that the supply is unlimited
    pub max_quantity: u32,
}

/// Represents a desired part
///
/// Assumed 1D for now
#[derive(Debug)]
pub struct Part {
    pub material: String,
    pub length: f32,
    pub quantity: u32,
}

/// Represents a set of of cuts to perform on an object
///
/// Assumed 1D for now
#[derive(Debug)]
pub struct CutList {
    pub material: String,

    /// The original length before cutting
    pub length: f32,

    pub cuts: Vec<f32>,
}
