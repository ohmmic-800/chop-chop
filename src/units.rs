use fraction::Fraction;

const FEET_TO_METERS_NUM: u64 = 3048;
const FEET_TO_METERS_DEN: u64 = 10000;

#[derive(PartialEq)]
pub enum LengthUnit {
    FeetInches,
    Inches,
    Centimeters,
    Meters,
}

impl LengthUnit {
    pub fn has_subunit(&self) -> bool {
        match *self {
            Self::FeetInches => true,
            _ => false,
        }
    }

    pub fn main_symbol(&self) -> &str {
        match *self {
            Self::FeetInches => "ft",
            Self::Inches => "in",
            Self::Centimeters => "cm",
            Self::Meters => "m",
        }
    }

    pub fn subunit_symbol(&self) -> &str {
        match *self {
            Self::FeetInches => "in",
            _ => "",
        }
    }
}

pub fn to_meters(length: Fraction, sublength: Fraction, unit: LengthUnit) -> Fraction {
    let feet_to_meters = Fraction::new(FEET_TO_METERS_NUM, FEET_TO_METERS_DEN);
    match unit {
        LengthUnit::FeetInches => feet_to_meters * (length + sublength * 12),
        LengthUnit::Inches => feet_to_meters * sublength * 12,
        LengthUnit::Meters => length,
        LengthUnit::Centimeters => length / 100,
    }
}
