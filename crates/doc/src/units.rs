//! Parsing and formatting lengths the way SketchUp's Measurements box does. Lengths are inches.
//!
//! Accepted: `1.5`, `1.5"`, `1.5in`, `1 1/2`, `1-1/2"`, `3/4`, `3'`, `3ft`, `3'6"`, `3' 6 1/2"`,
//! `38mm`, `3.8cm`, `1.2m`, and a leading `-` to flip direction. A comma is a decimal point too
//! (`38,5`). Bare numbers are in the units chosen ([`set_units`]): inches, or millimeters;
//! lengths are shown in them too.

use std::cell::Cell;
use std::fmt;

use serde::{Deserialize, Serialize};

/// How lengths are shown, and what a bare number means.
// Lowercase: the `.woodsketch` header has written `"units": "inches"` since the first version,
// so the spelling is part of the file format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    /// Fractional inches, to the nearest 1/16".
    #[default]
    Inches,
    /// Millimeters, to the nearest 0.1.
    Millimeters,
}

impl Units {
    pub const ALL: [Self; 2] = [Self::Inches, Self::Millimeters];
    pub fn name(self) -> &'static str {
        match self {
            Self::Inches => "Inches",
            Self::Millimeters => "Millimeters",
        }
    }
}

thread_local! {
    // Per thread: the app's (one window) thread sets it; tests running side by side don't share it.
    static UNITS: Cell<Units> = const { Cell::new(Units::Inches) };
}

/// Show lengths in `units`, and read bare numbers as them (on this thread).
pub fn set_units(units: Units) {
    UNITS.with(|u| u.set(units));
}

pub fn units() -> Units {
    UNITS.with(Cell::get)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseLengthError(String);

impl fmt::Display for ParseLengthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid length: \"{}\"", self.0)
    }
}

impl std::error::Error for ParseLengthError {}

/// Parse one length, in inches.
pub fn parse_length(input: &str) -> Result<f64, ParseLengthError> {
    let error = || ParseLengthError(input.trim().to_owned());
    let s = input.trim();
    let (sign, s) = match s.strip_prefix('-') {
        Some(rest) => (-1.0, rest.trim_start()),
        None => (1.0, s.strip_prefix('+').unwrap_or(s).trim_start()),
    };
    let inches = if let Some((feet, rest)) = s.split_once('\'') {
        let rest = rest.trim();
        let rest = rest.strip_suffix('"').or_else(|| rest.strip_suffix("in")).unwrap_or(rest);
        let inches = if rest.trim().is_empty() { 0.0 } else { parse_number(rest).ok_or_else(error)? };
        parse_number(feet).ok_or_else(error)? * 12.0 + inches
    } else {
        // Longest suffixes first, so "mm" isn't read as "m".
        const UNITS: [(&str, f64); 6] =
            [("mm", 1.0 / 25.4), ("cm", 1.0 / 2.54), ("in", 1.0), ("ft", 12.0), ("\"", 1.0), ("m", 1.0 / 0.0254)];
        let (number, scale) = UNITS
            .iter()
            .find_map(|(suffix, scale)| Some((s.strip_suffix(suffix)?, *scale)))
            .unwrap_or((s, if units() == Units::Millimeters { 1.0 / 25.4 } else { 1.0 }));
        parse_number(number).ok_or_else(error)? * scale
    };
    Ok(sign * inches)
}

/// Parse a comma- or semicolon-separated list, like `1.5",3.5"`. Empty entries are `None`,
/// meaning "keep the current value" (`,3.5` changes only the second dimension).
///
/// A semicolon anywhere in the input says *that* is the separator, which leaves the comma free
/// to be a decimal point: `5,5';6'` and `5.5',6'` are the same two lengths. Without a semicolon
/// the comma separates, as it always has, so `3,5` is still two lengths and not three and a
/// half -- which is why someone writing decimals with a comma needs the semicolon to say so.
pub fn parse_lengths(input: &str) -> Result<Vec<Option<f64>>, ParseLengthError> {
    split_values(input)
        .iter()
        .map(|part| if part.is_empty() { Ok(None) } else { parse_length(part).map(Some) })
        .collect()
}

/// Split typed input into its values: a semicolon anywhere makes the semicolon the separator and
/// every comma a decimal point, otherwise the comma separates. Each part comes back trimmed with
/// its comma already a point, so a plain `f64` parse reads it -- which is what Scale's factors
/// need, and the reason this is one function rather than the same two lines in two places.
pub fn split_values(input: &str) -> Vec<String> {
    let separator = if input.contains(';') { ';' } else { ',' };
    input.split(separator).map(|part| part.trim().replace(',', ".")).collect()
}

/// Format inches as fractional inches to the nearest 1/16", like `3 1/2"`. Rounded values get
/// SketchUp's `~` prefix.
pub fn format_length(inches: f64) -> String {
    // Millimeters to the nearest 0.1 (`88.9 mm`), when chosen.
    if units() == Units::Millimeters {
        let tenths = inches * 254.0;
        let exact = (tenths - tenths.round()).abs() < 1e-6;
        let mm = tenths.round() / 10.0;
        let approx = if exact { "" } else { "~ " };
        return if mm.fract() == 0.0 { format!("{approx}{mm:.0} mm") } else { format!("{approx}{mm:.1} mm") };
    }
    const DENOMINATOR: i64 = 16;
    let sixteenths = (inches.abs() * DENOMINATOR as f64).round();
    let exact = (inches.abs() * DENOMINATOR as f64 - sixteenths).abs() < 1e-6;
    let sixteenths = sixteenths as i64;
    let (whole, mut num, mut den) = (sixteenths / DENOMINATOR, sixteenths % DENOMINATOR, DENOMINATOR);
    while num > 0 && num % 2 == 0 {
        num /= 2;
        den /= 2;
    }
    let magnitude = match (whole, num) {
        (w, 0) => format!("{w}\""),
        (0, n) => format!("{n}/{den}\""),
        (w, n) => format!("{w} {n}/{den}\""),
    };
    let sign = if inches < 0.0 && sixteenths > 0 { "-" } else { "" };
    let approx = if exact { "" } else { "~ " };
    format!("{approx}{sign}{magnitude}")
}

/// A plain number: decimal (`1.5`), fraction (`3/4`) or mixed (`1 1/2`, `1-1/2`).
fn parse_number(s: &str) -> Option<f64> {
    let s = s.trim();
    let decimal = |t: &str| -> Option<f64> {
        // Half the world writes 5,5 for five and a half. Taken here rather than in the callers
        // so it holds for feet and inches, millimeters and the whole part of a fraction alike.
        let t = t.trim().replace(',', ".");
        let t = t.trim();
        let valid = !t.is_empty()
            && t != "."
            && t.chars().all(|c| c.is_ascii_digit() || c == '.')
            && t.matches('.').count() <= 1;
        if valid { t.parse().ok() } else { None }
    };
    let Some((numerator, denominator)) = s.rsplit_once('/') else { return decimal(s) };
    let (whole, numerator) = match numerator.trim().rsplit_once([' ', '-']) {
        Some((whole, n)) => (decimal(whole)?, n),
        None => (0.0, numerator),
    };
    let denominator = decimal(denominator)?;
    if denominator == 0.0 {
        return None;
    }
    Some(whole + decimal(numerator)? / denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> f64 {
        parse_length(s).unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn inches() {
        assert_eq!(parse("1.5"), 1.5);
        assert_eq!(parse("1.5\""), 1.5);
        assert_eq!(parse(" 3.5in "), 3.5);
        assert_eq!(parse(".75"), 0.75);
    }

    #[test]
    fn fractions() {
        assert_eq!(parse("3/4"), 0.75);
        assert_eq!(parse("1 1/2\""), 1.5);
        assert_eq!(parse("1-1/2"), 1.5);
        assert_eq!(parse("3 1/2"), 3.5);
    }

    #[test]
    fn feet_and_inches() {
        assert_eq!(parse("8'"), 96.0);
        assert_eq!(parse("8ft"), 96.0);
        assert_eq!(parse("3'6\""), 42.0);
        assert_eq!(parse("3' 6 1/2\""), 42.5);
        assert_eq!(parse("3'6"), 42.0);
    }

    #[test]
    fn metric() {
        assert!((parse("38mm") - 1.496_062_992).abs() < 1e-9);
        assert!((parse("2.54cm") - 1.0).abs() < 1e-12);
        assert!((parse("1m") - 39.370_078_74).abs() < 1e-6);
    }

    #[test]
    fn a_comma_is_a_decimal_point() {
        // Portuguese, Spanish, French, German: 5,5 is five and a half. One value on its own has
        // nothing to separate, so the comma can only be the decimal point.
        assert_eq!(parse("1,5"), 1.5);
        assert_eq!(parse("1,5\""), 1.5);
        assert_eq!(parse("5,5'"), 66.0);
        assert!((parse("38,5mm") - 38.5 / 25.4).abs() < 1e-9);
        assert_eq!(parse("1 1/2"), parse("1,5"));
    }

    #[test]
    fn a_semicolon_says_the_comma_is_a_decimal_point() {
        // The reason the semicolon is worth accepting: without it there is no way to write a
        // list of lengths that have decimals, because the comma would be doing both jobs.
        assert_eq!(parse_lengths("5,5';6'").unwrap(), parse_lengths("5.5',6'").unwrap());
        assert_eq!(parse_lengths("5,5';6'").unwrap(), vec![Some(66.0), Some(72.0)]);
        assert_eq!(parse_lengths("38,5;10;2,25").unwrap().len(), 3);
    }

    #[test]
    fn without_a_semicolon_the_comma_still_separates() {
        // Unchanged, and the whole reason the semicolon has to be typed: `3,5` has meant two
        // lengths since the first version, and drawings are full of people meaning that.
        assert_eq!(parse_lengths("3,5").unwrap(), vec![Some(3.0), Some(5.0)]);
        assert_eq!(parse_lengths("1.5\",3.5\"").unwrap(), vec![Some(1.5), Some(3.5)]);
        assert_eq!(parse_lengths(",3.5").unwrap(), vec![None, Some(3.5)]);
    }

    #[test]
    fn two_decimal_points_in_one_number_is_still_wrong() {
        assert!(parse_length("5,5.5").is_err());
        assert!(parse_lengths("5,5,5;6").is_err());
    }

    #[test]
    fn negative_flips() {
        assert_eq!(parse("-1.5\""), -1.5);
        assert_eq!(parse("-1 1/2"), -1.5);
    }

    #[test]
    fn rejects_garbage() {
        for bad in ["", "abc", "1..5", "1/0", ".", "inf", "NaN", "1.5x", "--2"] {
            assert!(parse_length(bad).is_err(), "{bad:?} should not parse");
        }
    }

    #[test]
    fn dimension_lists() {
        assert_eq!(parse_lengths("1.5\",3.5\"").unwrap(), [Some(1.5), Some(3.5)]);
        assert_eq!(parse_lengths("1 1/2; 3 1/2").unwrap(), [Some(1.5), Some(3.5)]);
        assert_eq!(parse_lengths(",3.5").unwrap(), [None, Some(3.5)]);
        assert!(parse_lengths("1.5,x").is_err());
    }

    #[test]
    fn formatting() {
        assert_eq!(format_length(1.5), "1 1/2\"");
        assert_eq!(format_length(3.5), "3 1/2\"");
        assert_eq!(format_length(96.0), "96\"");
        assert_eq!(format_length(0.0625), "1/16\"");
        assert_eq!(format_length(0.0), "0\"");
        assert_eq!(format_length(-0.75), "-3/4\"");
        assert_eq!(format_length(1.0 / 3.0), "~ 5/16\"");
    }
}

#[cfg(test)]
mod metric_tests {
    use super::*;

    #[test]
    fn millimeters_show_and_read_bare_numbers_as_millimeters() {
        set_units(Units::Millimeters);
        assert_eq!(format_length(3.5), "88.9 mm");
        assert_eq!(format_length(0.75), "~ 19.1 mm", "19.05 mm, rounded");
        assert_eq!(format_length(1.0 / 25.4 * 19.0), "19 mm");
        assert!((parse_length("19").unwrap() - 19.0 / 25.4).abs() < 1e-12, "a bare number is millimeters");
        assert_eq!(parse_length("3/4\"").unwrap(), 0.75, "inches still say so");
        assert!((parse_length(&format_length(1.0 / 25.4 * 88.9)).unwrap() - 88.9 / 25.4).abs() < 1e-9, "what's shown reads back");
        set_units(Units::Inches);
        assert_eq!(format_length(3.5), "3 1/2\"");
        assert_eq!(parse_length("19").unwrap(), 19.0);
    }
}
