// Integration tests for the duration parser using extracted test data
//
// These tests verify the duration parser implementation against test cases
// extracted from the official Prometheus and HPE Rust parser test suites.

#[path = "lexer/duration_tests.rs"]
mod duration_tests;

use rusty_promql_parser::lexer::duration::duration;

/// Helper to test that a duration parses to the expected milliseconds
fn assert_duration_parses(input: &str, expected_ms: i64) {
    let result = duration(input);
    match result {
        Ok((remaining, dur)) => {
            assert!(
                remaining.is_empty(),
                "Parser did not consume entire input '{}', remaining: '{}'",
                input,
                remaining
            );
            assert_eq!(
                dur.as_millis(),
                expected_ms,
                "For input '{}', expected {}ms, got {}ms",
                input,
                expected_ms,
                dur.as_millis()
            );
        }
        Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
    }
}

#[test]
fn test_valid_simple_durations_from_test_data() {
    for (input, expected_ms) in duration_tests::VALID_SIMPLE_DURATIONS {
        assert_duration_parses(input, *expected_ms);
    }
}

#[test]
fn test_valid_compound_durations_from_test_data() {
    for (input, expected_ms) in duration_tests::VALID_COMPOUND_DURATIONS {
        assert_duration_parses(input, *expected_ms);
    }
}

#[test]
fn test_duration_units_from_test_data() {
    // Verify each unit's millisecond value
    for (unit_str, expected_ms) in duration_tests::DURATION_UNITS {
        let input = format!("1{}", unit_str);
        assert_duration_parses(&input, *expected_ms);
    }
}

#[test]
fn test_range_selector_durations_from_test_data() {
    // These test cases have the duration inside brackets [5m]
    // We just parse the inner duration part
    for (input, expected_ms) in duration_tests::RANGE_SELECTOR_DURATIONS {
        // Extract the duration from inside brackets
        let inner = input.trim_start_matches('[').trim_end_matches(']');
        assert_duration_parses(inner, *expected_ms);
    }
}

#[test]
fn test_subquery_durations_from_test_data() {
    // Test subquery range durations (the part before the colon)
    for (input, range_ms, _step_ms) in duration_tests::SUBQUERY_DURATIONS {
        // Extract just the range part: "[10m:6s]" -> "10m" or "[10m5s:1h6ms]" -> "10m5s"
        let inner = input.trim_start_matches('[').trim_end_matches(']');
        let range_part = inner.split(':').next().unwrap();
        if !range_part.is_empty() {
            assert_duration_parses(range_part, *range_ms);
        }
    }
}

#[test]
fn test_subquery_step_durations_from_test_data() {
    // Test subquery step durations (the part after the colon)
    for (input, _range_ms, step_ms) in duration_tests::SUBQUERY_DURATIONS {
        if let Some(expected) = step_ms {
            // Extract just the step part: "[10m:6s]" -> "6s"
            let inner = input.trim_start_matches('[').trim_end_matches(']');
            let parts: Vec<&str> = inner.split(':').collect();
            if parts.len() > 1 && !parts[1].is_empty() {
                assert_duration_parses(parts[1], *expected);
            }
        }
    }
}

#[test]
fn test_lexer_durations_from_test_data() {
    for (input, expected_token) in duration_tests::LEXER_DURATION_TESTS {
        // The expected_token is the duration string
        // Parse it to verify it's valid
        let result = duration(expected_token);
        assert!(
            result.is_ok(),
            "Failed to parse expected duration '{}' from input '{}'",
            expected_token,
            input
        );
    }
}
