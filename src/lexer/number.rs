// Number literal parser for PromQL
//
// Supports:
// - Integers: 42, 0, 123
// - Floats: 3.14, .5, 5.
// - Hexadecimal: 0x1F, 0X2A
// - Octal: 0755 (leading zero)
// - Scientific notation: 1e10, 2.5E-3
// - Special values: Inf, +Inf, -Inf, NaN (case-insensitive)
// - Signed numbers: +42, -3.14

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while1},
    character::complete::{char, one_of},
    combinator::{map, map_res, opt, recognize, value},
    sequence::{pair, preceded},
};

/// Parse a PromQL number literal and return its f64 value.
///
/// This parser handles all PromQL number formats:
/// - Decimal integers and floats
/// - Hexadecimal (0x/0X prefix)
/// - Octal (leading 0)
/// - Scientific notation (e/E)
/// - Special values (Inf, NaN)
pub fn number(input: &str) -> IResult<&str, f64> {
    alt((special_float, signed_number)).parse(input)
}

/// Check if the next character is alphanumeric or underscore (identifier continuation)
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Parse special float values: Inf, +Inf, -Inf, NaN (case-insensitive)
/// These must not be followed by alphanumeric characters (to avoid matching "info" as "inf")
fn special_float(input: &str) -> IResult<&str, f64> {
    let (rest, val) = alt((
        value(f64::INFINITY, preceded(opt(char('+')), tag_no_case("Inf"))),
        value(f64::NEG_INFINITY, preceded(char('-'), tag_no_case("Inf"))),
        value(f64::NAN, preceded(opt(one_of("+-")), tag_no_case("NaN"))),
    ))
    .parse(input)?;

    // Ensure not followed by alphanumeric/underscore (would make it an identifier like "info")
    if rest.chars().next().is_some_and(is_ident_char) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    Ok((rest, val))
}

/// Parse a signed number (with optional +/- prefix)
fn signed_number(input: &str) -> IResult<&str, f64> {
    map(
        pair(opt(one_of("+-")), unsigned_number),
        |(sign, value)| match sign {
            Some('-') => -value,
            _ => value,
        },
    )
    .parse(input)
}

/// Parse an unsigned number (hex, octal, or decimal)
fn unsigned_number(input: &str) -> IResult<&str, f64> {
    alt((
        hexadecimal,
        // Modern octal with 0o/0O prefix
        octal_prefixed,
        // Legacy octal must come before decimal since both start with digits
        // But we need to be careful: "0.5" should parse as decimal, not octal
        octal_legacy,
        decimal_float,
    ))
    .parse(input)
}

/// Parse a hexadecimal number: 0x1F, 0X2A
fn hexadecimal(input: &str) -> IResult<&str, f64> {
    map_res(
        preceded(
            alt((tag("0x"), tag("0X"))),
            take_while1(|c: char| c.is_ascii_hexdigit()),
        ),
        |digits: &str| i64::from_str_radix(digits, 16).map(|v| v as f64),
    )
    .parse(input)
}

/// Parse a modern octal number with prefix: 0o755, 0O755
fn octal_prefixed(input: &str) -> IResult<&str, f64> {
    map_res(
        preceded(
            alt((tag("0o"), tag("0O"))),
            take_while1(|c: char| matches!(c, '0'..='7')),
        ),
        |digits: &str| i64::from_str_radix(digits, 8).map(|v| v as f64),
    )
    .parse(input)
}

/// Parse a legacy octal number: 0755
/// Only matches if it starts with 0 followed by octal digits (0-7)
/// and doesn't look like a decimal float (no dot or e/E following)
fn octal_legacy(input: &str) -> IResult<&str, f64> {
    // Must start with 0, followed by at least one octal digit
    // We need to be careful not to consume "0" alone or "0.5" or "0e5"
    let (remaining, _) = char('0')(input)?;

    // Check if there's at least one more octal digit
    if remaining.is_empty() {
        // Just "0" - let decimal handle it
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let next_char = remaining.chars().next().unwrap();

    // If next char is not an octal digit, let decimal handle it
    if !matches!(next_char, '0'..='7') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Parse the rest as octal
    let (remaining, octal_digits) = take_while1(|c: char| matches!(c, '0'..='7'))(remaining)?;

    // Make sure this isn't actually a decimal number (no dot or exponent)
    if let Some(c) = remaining.chars().next()
        && (c == '.' || c == 'e' || c == 'E')
    {
        // This is a decimal number, not octal
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Parse the octal value (including the initial 0)
    let full_octal = format!("0{}", octal_digits);
    match i64::from_str_radix(&full_octal, 8) {
        Ok(v) => Ok((remaining, v as f64)),
        Err(_) => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::MapRes,
        ))),
    }
}

/// Parse a decimal number (integer or float, with optional scientific notation)
fn decimal_float(input: &str) -> IResult<&str, f64> {
    map_res(recognize(decimal_float_inner), |s: &str| s.parse::<f64>()).parse(input)
}

/// Inner recognizer for decimal floats - captures the string representation
fn decimal_float_inner(input: &str) -> IResult<&str, &str> {
    alt((
        // Case 1: .42 or .42e10
        recognize((char('.'), decimal_digits, opt(exponent))),
        // Case 2: 42e10 or 42.42e10 (exponent required)
        recognize((
            decimal_digits,
            opt(pair(char('.'), opt(decimal_digits))),
            exponent,
        )),
        // Case 3: 42. or 42.42 (no exponent)
        recognize((decimal_digits, char('.'), opt(decimal_digits))),
        // Case 4: plain integer
        recognize(decimal_digits),
    ))
    .parse(input)
}

/// Parse decimal digits (one or more)
fn decimal_digits(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_digit())(input)
}

/// Parse the exponent part: e10, E-3, e+5
fn exponent(input: &str) -> IResult<&str, &str> {
    recognize((one_of("eE"), opt(one_of("+-")), decimal_digits)).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to test that a number parses to the expected value
    fn assert_number(input: &str, expected: f64) {
        let result = number(input);
        match result {
            Ok((remaining, value)) => {
                assert!(
                    remaining.is_empty(),
                    "Parser did not consume entire input '{}', remaining: '{}'",
                    input,
                    remaining
                );
                if expected.is_nan() {
                    assert!(
                        value.is_nan(),
                        "Expected NaN for input '{}', got {}",
                        input,
                        value
                    );
                } else {
                    assert!(
                        (value - expected).abs() < f64::EPSILON || value == expected,
                        "For input '{}', expected {}, got {}",
                        input,
                        expected,
                        value
                    );
                }
            }
            Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
        }
    }

    /// Helper to test that a number parses to the expected value with remaining input
    fn assert_number_partial(input: &str, expected: f64, expected_remaining: &str) {
        let result = number(input);
        match result {
            Ok((remaining, value)) => {
                assert_eq!(
                    remaining, expected_remaining,
                    "For input '{}', expected remaining '{}', got '{}'",
                    input, expected_remaining, remaining
                );
                if expected.is_nan() {
                    assert!(
                        value.is_nan(),
                        "Expected NaN for input '{}', got {}",
                        input,
                        value
                    );
                } else {
                    assert!(
                        (value - expected).abs() < f64::EPSILON || value == expected,
                        "For input '{}', expected {}, got {}",
                        input,
                        expected,
                        value
                    );
                }
            }
            Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
        }
    }

    /// Helper to test that input fails to parse as a number
    fn assert_not_number(input: &str) {
        let result = number(input);
        assert!(
            result.is_err(),
            "Expected '{}' to fail parsing, but got {:?}",
            input,
            result
        );
    }

    // Basic integers
    #[test]
    fn test_integer() {
        assert_number("1", 1.0);
        assert_number("0", 0.0);
        assert_number("42", 42.0);
        assert_number("123", 123.0);
    }

    // Floats
    #[test]
    fn test_float() {
        assert_number(".5", 0.5);
        assert_number("5.", 5.0);
        assert_number("123.4567", 123.4567);
        assert_number("4.23", 4.23);
        assert_number(".3", 0.3);
    }

    // Scientific notation
    #[test]
    fn test_scientific() {
        assert_number("5e-3", 0.005);
        assert_number("5e3", 5000.0);
        assert_number("5e+3", 5000.0);
        assert_number("1e10", 1e10);
        assert_number("2.5E-3", 0.0025);
        assert_number("1e1", 10.0);
        assert_number("1e-1", 0.1);
        assert_number("1.0e1", 10.0);
        assert_number("1e01", 10.0);
        assert_number("1E01", 10.0);
        assert_number("1.e2", 100.0);
    }

    // Hexadecimal
    #[test]
    fn test_hex() {
        assert_number("0xc", 12.0);
        assert_number("0x123", 291.0);
        assert_number("0X2A", 42.0);
        assert_number("0x1F", 31.0);
        assert_number("0xA", 10.0);
    }

    // Octal - both legacy (leading 0) and modern (0o prefix)
    #[test]
    fn test_octal() {
        // Legacy octal with leading zero (from Prometheus parse_test.go)
        assert_number("0755", 493.0); // 7*64 + 5*8 + 5 = 493
        assert_number("0644", 420.0); // 6*64 + 4*8 + 4 = 420
        assert_number("07", 7.0);
        assert_number("010", 8.0); // 1*8 + 0 = 8
        assert_number("0777", 511.0); // 7*64 + 7*8 + 7 = 511
    }

    #[test]
    fn test_octal_prefixed() {
        // Modern octal with 0o/0O prefix (Go 1.13+ strconv.ParseInt)
        assert_number("0o0", 0.0);
        assert_number("0O0", 0.0);
        assert_number("0o7", 7.0);
        assert_number("0o10", 8.0);
        assert_number("0o755", 493.0);
        assert_number("0o777", 511.0);
        assert_number("0O755", 493.0);
    }

    #[test]
    fn test_octal_signed() {
        // Signed octal (from Prometheus parse_test.go: -0755 = -493)
        assert_number("-0755", -493.0);
        assert_number("+0755", 493.0);
        assert_number("-0o755", -493.0);
        assert_number("+0o755", 493.0);
    }

    // Signed numbers
    #[test]
    fn test_signed() {
        assert_number("-0755", -493.0);
        assert_number("-1", -1.0);
        assert_number("+1", 1.0);
        assert_number("-1e1", -10.0);
        assert_number("-1e-1", -0.1);
        assert_number("+5.5e-3", 0.0055);
    }

    // Special float values
    #[test]
    fn test_special_floats() {
        assert_number("NaN", f64::NAN);
        assert_number("nAN", f64::NAN);
        assert_number("Inf", f64::INFINITY);
        assert_number("iNf", f64::INFINITY);
        assert_number("+Inf", f64::INFINITY);
        assert_number("-Inf", f64::NEG_INFINITY);
    }

    // Numbers followed by other content (partial parsing)
    #[test]
    fn test_partial_parse() {
        assert_number_partial("NaN 123", f64::NAN, " 123");
        assert_number_partial("123abc", 123.0, "abc");
    }

    // Invalid numbers - these should NOT parse as identifiers
    #[test]
    fn test_not_numbers() {
        // These start like numbers but aren't valid
        assert_not_number("."); // Just a dot
        assert_not_number(""); // Empty
    }

    // Edge cases that should parse correctly
    #[test]
    fn test_edge_cases() {
        // 0 alone should parse as 0, not octal
        assert_number("0", 0.0);
        // 0.5 should parse as decimal, not octal
        assert_number("0.5", 0.5);
        // 0e5 should parse as decimal with exponent
        assert_number("0e5", 0.0);
    }
}
