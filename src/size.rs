use fraction::{Fraction, Zero};
use serde::{Deserialize, Serialize};

const FEET_TO_METERS_NUM: u64 = 3048;
const FEET_TO_METERS_DEN: u64 = 10000;

pub enum FractionFormat {
    Decimal(usize),
    Fraction,
    Mixed,
}

impl FractionFormat {
    pub fn format(&self, fraction: Fraction) -> String {
        match *self {
            Self::Decimal(precision) => Self::format_decimal(fraction, precision),
            Self::Mixed => Self::format_mixed(fraction),
            Self::Fraction => format!("{}", fraction),
        }
    }

    fn format_mixed(fraction: Fraction) -> String {
        let int = fraction.trunc();
        let remainder = fraction - int;
        if int.is_zero() && fraction.is_zero() {
            String::from("0")
        } else if int.is_zero() {
            format!("{}", remainder)
        } else if remainder.is_zero() {
            format!("{}", int)
        } else {
            format!("{} {}", int, remainder)
        }
    }

    fn format_decimal(fraction: Fraction, precision: usize) -> String {
        // Convert to f64 to get correct rounding behavior
        let float: f64 = fraction.try_into().unwrap();

        let result = format!("{0:.1$}", float, precision);
        if precision > 0 {
            // Trim trailing zeros
            let parts: Vec<_> = result.split(".").collect();
            vec![parts[0], parts[1].trim_end_matches('0')]
                .join(".")
                .trim_end_matches(".")
                .to_string()
        } else {
            result
        }
    }
}

/// The `major` and `minor` fields allow splitting the size into e.g. feet and inches
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub unit: SizeUnit,
    pub major: Fraction,
    pub minor: Fraction,
}

impl Size {
    pub fn format(&self, fraction_format: &FractionFormat) -> String {
        let mut output = format!(
            "{} {}",
            fraction_format.format(self.major),
            self.unit.major_symbol()
        );
        if self.unit.has_minor() {
            output += &format!(
                ", {} {}",
                fraction_format.format(self.minor),
                self.unit.minor_symbol()
            );
        }
        output
    }

    pub fn from_meters<T>(meters: T) -> Self
    where
        Fraction: From<T>,
    {
        Self {
            unit: SizeUnit::Meters,
            major: Fraction::from(meters),
            minor: Fraction::zero(),
        }
    }

    pub fn to_meters(&self) -> Fraction {
        let feet_to_meters = Fraction::new(FEET_TO_METERS_NUM, FEET_TO_METERS_DEN);
        match self.unit {
            SizeUnit::FeetInches => feet_to_meters * (self.major + self.minor / 12),
            SizeUnit::Inches => feet_to_meters * (self.major / 12),
            SizeUnit::Meters => self.major,
            SizeUnit::Centimeters => self.major / 100,
        }
    }

    pub fn to_meters_f64(&self) -> f64 {
        self.to_meters().try_into().unwrap()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum SizeUnit {
    FeetInches,
    Inches,
    Centimeters,
    #[default]
    Meters,
}

impl SizeUnit {
    pub fn has_minor(&self) -> bool {
        match *self {
            Self::FeetInches => true,
            _ => false,
        }
    }

    pub fn major_name(&self) -> &'static str {
        match *self {
            Self::FeetInches => "Feet",
            Self::Inches => "Inches",
            Self::Centimeters => "Centimeters",
            Self::Meters => "Meters",
        }
    }

    pub fn major_symbol(&self) -> &'static str {
        match *self {
            Self::FeetInches => "ft",
            Self::Inches => "in",
            Self::Centimeters => "cm",
            Self::Meters => "m",
        }
    }

    pub fn minor_name(&self) -> &'static str {
        match *self {
            Self::FeetInches => "Inches",
            _ => "",
        }
    }

    pub fn minor_symbol(&self) -> &'static str {
        match *self {
            Self::FeetInches => "in",
            _ => "",
        }
    }
}
