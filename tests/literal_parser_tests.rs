// Integration tests for number and string literal parsing using extracted test data
//
// These tests verify the lexer's number and string parsing against test cases
// extracted from the official Prometheus parser test suites.

#[path = "parser/literal_tests.rs"]
mod literal_tests;

use rusty_promql_parser::lexer::number::number;
use rusty_promql_parser::lexer::string::string_literal;
use rusty_promql_parser::{Expr, expr};

// =============================================================================
// Integer Literals
// =============================================================================

#[test]
fn test_valid_integers_from_test_data() {
    for (input, expected) in literal_tests::VALID_INTEGERS {
        let result = number(input);
        assert!(result.is_ok(), "Failed to parse integer: '{}'", input);
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty(), "Did not consume entire input: '{}'", input);
        assert_eq!(value, *expected, "Value mismatch for '{}'", input);
    }
}

// =============================================================================
// Float Literals
// =============================================================================

#[test]
fn test_valid_floats_from_test_data() {
    for (input, expected) in literal_tests::VALID_FLOATS {
        let result = number(input);
        assert!(result.is_ok(), "Failed to parse float: '{}'", input);
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty(), "Did not consume entire input: '{}'", input);
        assert!(
            (value - expected).abs() < 1e-10,
            "Value mismatch for '{}': expected {}, got {}",
            input,
            expected,
            value
        );
    }
}

// =============================================================================
// Scientific Notation Literals
// =============================================================================

#[test]
fn test_valid_scientific_from_test_data() {
    for (input, expected) in literal_tests::VALID_SCIENTIFIC {
        let result = number(input);
        assert!(
            result.is_ok(),
            "Failed to parse scientific notation: '{}'",
            input
        );
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty(), "Did not consume entire input: '{}'", input);
        assert!(
            (value - expected).abs() < 1e-10 || (value / expected - 1.0).abs() < 1e-10,
            "Value mismatch for '{}': expected {}, got {}",
            input,
            expected,
            value
        );
    }
}

// =============================================================================
// Hexadecimal Literals
// =============================================================================

#[test]
fn test_valid_hex_from_test_data() {
    for (input, expected) in literal_tests::VALID_HEX {
        let result = number(input);
        assert!(result.is_ok(), "Failed to parse hex: '{}'", input);
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty(), "Did not consume entire input: '{}'", input);
        assert_eq!(value, *expected, "Value mismatch for '{}'", input);
    }
}

// =============================================================================
// Octal Literals
// =============================================================================

#[test]
fn test_valid_octal_from_test_data() {
    for (input, expected) in literal_tests::VALID_OCTAL {
        let result = number(input);
        assert!(result.is_ok(), "Failed to parse octal: '{}'", input);
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty(), "Did not consume entire input: '{}'", input);
        assert_eq!(value, *expected, "Value mismatch for '{}'", input);
    }
}

// =============================================================================
// Special Float Values (Inf, NaN)
// =============================================================================

#[test]
fn test_special_floats_from_test_data() {
    for (input, expected_type) in literal_tests::SPECIAL_FLOATS {
        let result = number(input);
        assert!(result.is_ok(), "Failed to parse special float: '{}'", input);
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty(), "Did not consume entire input: '{}'", input);
        match *expected_type {
            "inf" => {
                assert!(value.is_infinite(), "Expected Inf for '{}'", input);
                assert!(value > 0.0, "Expected positive Inf for '{}'", input);
            }
            "-inf" => {
                assert!(value.is_infinite(), "Expected -Inf for '{}'", input);
                assert!(value < 0.0, "Expected negative Inf for '{}'", input);
            }
            "nan" => {
                assert!(value.is_nan(), "Expected NaN for '{}'", input);
            }
            _ => panic!("Unknown special float type: {}", expected_type),
        }
    }
}

// =============================================================================
// Double-Quoted String Literals
// =============================================================================

#[test]
fn test_valid_double_quoted_strings_from_test_data() {
    for (input, expected) in literal_tests::VALID_DOUBLE_QUOTED_STRINGS {
        let result = string_literal(input);
        assert!(
            result.is_ok(),
            "Failed to parse double-quoted string: '{}'",
            input
        );
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty(), "Did not consume entire input: '{}'", input);
        assert_eq!(value.as_str(), *expected, "Value mismatch for '{}'", input);
    }
}

// =============================================================================
// Single-Quoted String Literals
// =============================================================================

#[test]
fn test_valid_single_quoted_strings_from_test_data() {
    for (input, expected) in literal_tests::VALID_SINGLE_QUOTED_STRINGS {
        let result = string_literal(input);
        assert!(
            result.is_ok(),
            "Failed to parse single-quoted string: '{}'",
            input
        );
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty(), "Did not consume entire input: '{}'", input);
        assert_eq!(value.as_str(), *expected, "Value mismatch for '{}'", input);
    }
}

// =============================================================================
// Raw (Backtick) String Literals
// =============================================================================

#[test]
fn test_valid_backtick_strings_from_test_data() {
    for (input, expected) in literal_tests::VALID_BACKTICK_STRINGS {
        let result = string_literal(input);
        assert!(
            result.is_ok(),
            "Failed to parse backtick string: '{}'",
            input
        );
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty(), "Did not consume entire input: '{}'", input);
        assert_eq!(value.as_str(), *expected, "Value mismatch for '{}'", input);
    }
}

// =============================================================================
// Expression Parser - Numbers and Strings
// =============================================================================

#[test]
#[allow(clippy::approx_constant)]
fn test_expr_parses_numbers() {
    let test_cases = [
        ("42", 42.0),
        ("3.14", 3.14),
        ("1e10", 1e10),
        ("0xFF", 255.0),
    ];
    for (input, expected) in test_cases {
        let result = expr(input);
        assert!(
            result.is_ok(),
            "Failed to parse number literal: '{}'",
            input
        );
        let (rest, e) = result.unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Number(n) => assert_eq!(n, expected),
            _ => panic!("Expected Expr::Number for '{}'", input),
        }
    }
}

#[test]
fn test_expr_parses_strings() {
    let test_cases = [
        (r#""hello""#, "hello"),
        ("'world'", "world"),
        ("`raw`", "raw"),
    ];
    for (input, expected) in test_cases {
        let result = expr(input);
        assert!(
            result.is_ok(),
            "Failed to parse string literal: '{}'",
            input
        );
        let (rest, e) = result.unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::String(s) => assert_eq!(s.as_str(), expected),
            _ => panic!("Expected Expr::String for '{}'", input),
        }
    }
}

// =============================================================================
// Display/Formatting
// =============================================================================

#[test]
#[allow(clippy::approx_constant)]
fn test_number_display_roundtrip() {
    // Test that Expr::Number Display produces valid PromQL that can be re-parsed
    let numbers = [42.0, 3.14, 1e10, f64::INFINITY, f64::NEG_INFINITY];
    for n in numbers {
        let e = Expr::Number(n);
        let displayed = format!("{}", e);
        // The displayed string should be parseable
        let result = number(&displayed);
        assert!(
            result.is_ok(),
            "Display output '{}' should be parseable",
            displayed
        );
    }
}
