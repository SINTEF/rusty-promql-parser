// Integration tests for the string parser using extracted test data
//
// These tests verify the string parser implementation against test cases
// extracted from the official Prometheus and HPE Rust parser test suites.

#[path = "lexer/string_tests.rs"]
mod string_tests;

use rusty_promql_parser::lexer::string::string_literal;

/// Helper to test that a string parses to the expected value
fn assert_string_parses(input: &str, expected: &str) {
    let result = string_literal(input);
    match result {
        Ok((remaining, value)) => {
            assert!(
                remaining.is_empty(),
                "Parser did not consume entire input '{}', remaining: '{}'",
                input,
                remaining
            );
            assert_eq!(
                value, expected,
                "For input '{}', expected {:?}, got {:?}",
                input, expected, value
            );
        }
        Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
    }
}

/// Helper to test that input fails to parse or doesn't consume entirely
fn assert_string_fails(input: &str) {
    let result = string_literal(input);
    match result {
        Ok((remaining, _)) => {
            assert!(
                !remaining.is_empty(),
                "Expected '{}' to fail or not fully parse, but it parsed completely",
                input
            );
        }
        Err(_) => {
            // Good - it failed to parse
        }
    }
}

#[test]
fn test_valid_double_quoted_from_test_data() {
    for (input, expected) in string_tests::VALID_DOUBLE_QUOTED {
        assert_string_parses(input, expected);
    }
}

#[test]
fn test_valid_single_quoted_from_test_data() {
    for (input, expected) in string_tests::VALID_SINGLE_QUOTED {
        assert_string_parses(input, expected);
    }
}

#[test]
fn test_valid_raw_strings_from_test_data() {
    for (input, expected) in string_tests::VALID_RAW_STRINGS {
        assert_string_parses(input, expected);
    }
}

#[test]
fn test_label_matcher_strings_from_test_data() {
    for (input, expected) in string_tests::LABEL_MATCHER_STRINGS {
        assert_string_parses(input, expected);
    }
}

#[test]
fn test_invalid_strings_from_test_data() {
    // Note: Some invalid cases in the test data are about error messages,
    // not just whether parsing fails. We just verify they don't parse successfully.
    for (input, _expected_error) in string_tests::INVALID_STRINGS {
        assert_string_fails(input);
    }
}
