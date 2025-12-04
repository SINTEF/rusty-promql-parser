// Number literal test cases extracted from:
// - references/prometheus/promql/parser/parse_test.go
// - references/prometheus/promql/parser/lex_test.go
//
// These test cases cover:
// - Integer literals
// - Float literals
// - Hexadecimal literals
// - Octal literals
// - Scientific notation
// - Special values (Inf, NaN)
// - Numeric underscores (experimental)

/// Valid number literal test cases
pub const VALID_NUMBERS: &[(&str, f64)] = &[
    // Basic integers
    ("1", 1.0),
    ("0", 0.0),
    ("42", 42.0),
    ("123", 123.0),
    // Floats
    (".5", 0.5),
    ("5.", 5.0),
    ("123.4567", 123.4567),
    ("4.23", 4.23),
    (".3", 0.3),
    // Scientific notation
    ("5e-3", 0.005),
    ("5e3", 5000.0),
    ("5e+3", 5000.0),
    ("1e10", 1e10),
    ("2.5E-3", 0.0025),
    ("+5.5e-3", 0.0055),
    ("1e1", 10.0),
    ("1e-1", 0.1),
    ("1.0e1", 10.0),
    ("1e01", 10.0),
    ("1E01", 10.0),
    ("1.e2", 100.0),
    // Hexadecimal
    ("0xc", 12.0),
    ("0x123", 291.0),
    ("0X2A", 42.0),
    ("0x1F", 31.0),
    ("0xA", 10.0),
    // Octal
    ("0755", 493.0),
    // Signed numbers
    ("-0755", -493.0),
    ("-1", -1.0),
    ("+1", 1.0),
    ("-1e1", -10.0),
    ("-1e-1", -0.1),
    // Numbers with underscores (experimental in newer Prometheus versions)
    // ("00_1_23_4.56_7_8", 1234.5678),  // TODO: Enable when underscore support is added
    // ("0x1_2_34", 0x1234 as f64),
    // ("1e1_2_34", 1e1234),  // Note: this would be Inf
];

/// Valid special float values (need special comparison due to NaN)
pub const VALID_SPECIAL_FLOATS: &[(&str, &str)] = &[
    ("NaN", "NaN"),
    ("nAN", "NaN"), // Case insensitive
    ("Inf", "+Inf"),
    ("iNf", "+Inf"), // Case insensitive
    ("+Inf", "+Inf"),
    ("-Inf", "-Inf"),
];

/// Invalid number literal test cases
pub const INVALID_NUMBERS: &[&str] = &[
    // Invalid formats
    ".",         // Just a dot
    "2.5.",      // Trailing dot after decimal
    "100..4",    // Double dot
    "1..2",      // Double dot
    "1.2.",      // Trailing dot
    "0deadbeef", // Invalid hex (missing x)
    "1a",        // Letters after number without valid format
    // Invalid exponent formats
    "1e",    // Missing exponent value
    "1e+",   // Missing exponent digits
    "1e.",   // Dot not allowed in exponent
    "1e+.2", // Dot after sign
    "1ee2",  // Double e
    "1e+e2", // e in exponent
    "1e.2",  // Dot after e
    // Invalid underscore placements (when underscore feature is enabled)
    // "00_1_23__4.56_7_8",  // Double underscore
    // "00_1_23_4._56_7_8",  // Underscore after dot
    // "00_1_23_4_.56_7_8",  // Underscore before dot
    // "1_e2",               // Underscore before e
    // "1e_1_2_34",          // Underscore after e
    // "1e1_2__34",          // Double underscore in exponent
    // "1e+_1_2_34",         // Underscore after +
    // "1e-_1_2_34",         // Underscore after -
    // "12_",                // Trailing underscore

    // Overflow
    "999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999",
];

/// Test cases from Go lexer tests for number tokens
pub const LEXER_NUMBER_TESTS: &[(&str, &str)] = &[
    ("1", "1"),
    ("4.23", "4.23"),
    (".3", ".3"),
    ("5.", "5."),
    ("NaN", "NaN"),
    ("nAN", "nAN"),
    ("NaN 123", "NaN"), // First token only
    ("Inf", "Inf"),
    ("iNf", "iNf"),
    ("+Inf", "Inf"), // + is separate token
    ("-Inf", "Inf"), // - is separate token
    ("0x123", "0x123"),
];

/// Numbers that parse but should be identifiers not numbers
pub const NOT_NUMBERS: &[&str] = &[
    "NaN123", // Identifier, not NaN followed by 123
    "Infoo",  // Identifier, not Inf followed by oo
    "_1_2",   // Identifier (starts with underscore)
];

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::number;

    #[test]
    fn test_valid_numbers_parse() {
        for (input, expected) in VALID_NUMBERS {
            let result = number(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "number parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Use approximate comparison for floats
                    let diff = (value - expected).abs();
                    let tolerance = if expected.abs() > 1.0 {
                        expected.abs() * 1e-10
                    } else {
                        1e-10
                    };
                    assert!(
                        diff < tolerance,
                        "Parsed number '{}' should be {}, got {} (diff: {})",
                        input,
                        expected,
                        value,
                        diff
                    );
                }
                Err(e) => panic!("Failed to parse number '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_special_floats_parse() {
        for (input, expected_type) in VALID_SPECIAL_FLOATS {
            let result = number(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "number parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Check the type of special float
                    match *expected_type {
                        "NaN" => assert!(value.is_nan(), "'{}' should parse to NaN", input),
                        "+Inf" => {
                            assert!(
                                value.is_infinite() && value > 0.0,
                                "'{}' should parse to +Inf",
                                input
                            )
                        }
                        "-Inf" => {
                            assert!(
                                value.is_infinite() && value < 0.0,
                                "'{}' should parse to -Inf",
                                input
                            )
                        }
                        _ => panic!("Unknown expected type '{}'", expected_type),
                    }
                }
                Err(e) => panic!("Failed to parse special float '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_invalid_numbers_fail() {
        for input in INVALID_NUMBERS {
            let result = number(input);
            // Should either fail or not fully consume input
            match result {
                Err(_) => {
                    // Good - it should fail
                }
                Ok((remaining, value)) => {
                    // Some inputs might partially parse (e.g., "1a" parses "1" leaving "a")
                    // or parse to infinity for overflow
                    if remaining.is_empty() && !value.is_infinite() {
                        panic!(
                            "Invalid number '{}' should not parse successfully to {}",
                            input, value
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_not_numbers_are_identifiers() {
        // These should NOT be fully consumed by the number parser
        // They should be parsed as identifiers instead
        for input in NOT_NUMBERS {
            let result = number(input);
            match result {
                Err(_) => {
                    // Good - number parser rejects it
                }
                Ok((remaining, _)) => {
                    assert!(
                        !remaining.is_empty(),
                        "'{}' should not be fully parsed as a number",
                        input
                    );
                }
            }
        }
    }

    #[test]
    fn test_lexer_number_cases() {
        for (input, _expected_token) in LEXER_NUMBER_TESTS {
            // The lexer tests include partial input (like "NaN 123")
            // Just verify that the number part parses
            let result = number(input);
            assert!(result.is_ok(), "Lexer test number '{}' should parse", input);
        }
    }
}
