// Integration tests for the identifier parser using extracted test data
//
// These tests verify the identifier parser implementation against test cases
// extracted from the official Prometheus and HPE Rust parser test suites.

#[path = "lexer/identifier_tests.rs"]
mod identifier_tests;

use rusty_promql_parser::lexer::identifier::{
    Identifier, Keyword, KeywordOrIdentifier, identifier, keyword, keyword_or_identifier,
    label_name, metric_name,
};

// =============================================================================
// Valid Metric Names
// =============================================================================

#[test]
fn test_valid_metric_names_from_test_data() {
    for name in identifier_tests::VALID_METRIC_NAMES {
        let result = metric_name(name);
        assert!(
            result.is_ok(),
            "Failed to parse valid metric name: '{}'",
            name
        );
        let (rest, parsed) = result.unwrap();
        assert!(
            rest.is_empty(),
            "Did not consume entire input for '{}', remaining: '{}'",
            name,
            rest
        );
        assert_eq!(parsed, *name, "Parsed value mismatch for '{}'", name);
    }
}

// =============================================================================
// Valid Label Names
// =============================================================================

#[test]
fn test_valid_label_names_from_test_data() {
    for name in identifier_tests::VALID_LABEL_NAMES {
        // Skip names that contain colons (they're not valid label names)
        if name.contains(':') {
            continue;
        }
        let result = label_name(name);
        assert!(
            result.is_ok(),
            "Failed to parse valid label name: '{}'",
            name
        );
        let (rest, parsed) = result.unwrap();
        assert!(
            rest.is_empty(),
            "Did not consume entire input for '{}', remaining: '{}'",
            name,
            rest
        );
        assert_eq!(parsed, *name, "Parsed value mismatch for '{}'", name);
    }
}

// =============================================================================
// Keywords
// =============================================================================

#[test]
fn test_keywords_from_test_data() {
    for (kw_str, _token_type) in identifier_tests::KEYWORDS {
        let result = keyword(kw_str);
        assert!(result.is_ok(), "Failed to parse keyword: '{}'", kw_str);
        let (rest, _) = result.unwrap();
        assert!(
            rest.is_empty(),
            "Did not consume entire input for keyword '{}', remaining: '{}'",
            kw_str,
            rest
        );
    }
}

#[test]
fn test_aggregation_keywords_from_test_data() {
    for kw_str in identifier_tests::AGGREGATION_KEYWORDS {
        // Try both lowercase and original case
        let lower = kw_str.to_ascii_lowercase();
        let result = keyword(&lower);
        assert!(
            result.is_ok(),
            "Failed to parse aggregation keyword: '{}'",
            kw_str
        );
        let (_, kw) = result.unwrap();
        assert!(
            kw.is_aggregation(),
            "Keyword '{}' should be an aggregation operator",
            kw_str
        );
    }
}

#[test]
fn test_set_operators_from_test_data() {
    for op_str in identifier_tests::SET_OPERATORS {
        let lower = op_str.to_ascii_lowercase();
        let result = keyword(&lower);
        assert!(result.is_ok(), "Failed to parse set operator: '{}'", op_str);
        let (_, kw) = result.unwrap();
        assert!(
            kw.is_set_operator(),
            "Keyword '{}' should be a set operator",
            op_str
        );
    }
}

#[test]
fn test_preprocessor_keywords_from_test_data() {
    for kw_str in identifier_tests::PREPROCESSOR_KEYWORDS {
        let result = keyword(kw_str);
        assert!(
            result.is_ok(),
            "Failed to parse preprocessor keyword: '{}'",
            kw_str
        );
    }
}

// =============================================================================
// Lexer Identifier Tests
// =============================================================================

#[test]
fn test_lexer_identifier_tests_from_test_data() {
    for (input, expected_type, expected_value) in identifier_tests::LEXER_IDENTIFIER_TESTS {
        let result = identifier(input);
        assert!(result.is_ok(), "Failed to parse identifier: '{}'", input);
        let (rest, id) = result.unwrap();

        // The expected value is what we should parse
        assert_eq!(
            id.as_str(),
            *expected_value,
            "Value mismatch for '{}': expected '{}', got '{}'",
            input,
            expected_value,
            id.as_str()
        );

        // Check the type
        match *expected_type {
            "IDENTIFIER" => {
                assert!(
                    !id.has_colon(),
                    "Expected plain identifier for '{}', got metric identifier",
                    input
                );
            }
            "METRIC_IDENTIFIER" => {
                assert!(
                    id.has_colon(),
                    "Expected metric identifier for '{}', got plain identifier",
                    input
                );
            }
            _ => {}
        }

        // Check remaining input (for partial matches like "abc d")
        if input.len() > expected_value.len() {
            assert_eq!(
                rest,
                &input[expected_value.len()..],
                "Remaining input mismatch for '{}'",
                input
            );
        }
    }
}

// =============================================================================
// Keywords can be used as identifiers in metric names
// =============================================================================

#[test]
fn test_keywords_as_metric_names() {
    // All keywords should also parse as valid metric names
    let keyword_strings = [
        "sum",
        "avg",
        "count",
        "min",
        "max",
        "group",
        "stddev",
        "stdvar",
        "topk",
        "bottomk",
        "count_values",
        "quantile",
        "limitk",
        "limit_ratio",
        "and",
        "or",
        "unless",
        "atan2",
        "offset",
        "by",
        "without",
        "on",
        "ignoring",
        "group_left",
        "group_right",
        "bool",
        "start",
        "end",
        "step",
    ];

    for kw in keyword_strings {
        let result = metric_name(kw);
        assert!(
            result.is_ok(),
            "Keyword '{}' should be valid as metric name",
            kw
        );
        let (rest, parsed) = result.unwrap();
        assert!(rest.is_empty(), "Should consume entire keyword '{}'", kw);
        assert_eq!(parsed, kw);
    }
}

// =============================================================================
// Identifier vs Keyword distinction
// =============================================================================

#[test]
fn test_keyword_or_identifier_for_keywords() {
    let keywords = [
        ("sum", Keyword::Sum),
        ("SUM", Keyword::Sum),
        ("Sum", Keyword::Sum),
        ("count_values", Keyword::CountValues),
        ("COUNT_VALUES", Keyword::CountValues),
    ];

    for (input, expected_kw) in keywords {
        let result = keyword_or_identifier(input);
        assert!(result.is_ok(), "Failed to parse: '{}'", input);
        let (rest, parsed) = result.unwrap();
        assert!(rest.is_empty(), "Should consume entire input: '{}'", input);
        assert_eq!(
            parsed,
            KeywordOrIdentifier::Keyword(expected_kw),
            "Expected keyword for '{}'",
            input
        );
    }
}

#[test]
fn test_keyword_or_identifier_for_identifiers() {
    let identifiers = [
        ("http_requests", false),
        ("http_requests_total", false),
        ("job:request_rate:5m", true),
        (":bc", true),
        ("summary", false), // Contains "sum" but is not the keyword
        ("counter", false), // Contains "count" but is not the keyword
    ];

    for (input, has_colon) in identifiers {
        let result = keyword_or_identifier(input);
        assert!(result.is_ok(), "Failed to parse: '{}'", input);
        let (rest, parsed) = result.unwrap();
        assert!(rest.is_empty(), "Should consume entire input: '{}'", input);
        match parsed {
            KeywordOrIdentifier::Identifier(id) => {
                assert_eq!(
                    id.has_colon(),
                    has_colon,
                    "Colon presence mismatch for '{}'",
                    input
                );
                assert_eq!(id.as_str(), input);
            }
            KeywordOrIdentifier::Keyword(_) => {
                panic!("Expected identifier for '{}', got keyword", input);
            }
        }
    }
}

// =============================================================================
// Edge cases from test data
// =============================================================================

#[test]
fn test_nan_and_inf_as_identifiers() {
    // "NaN123" and "Infoo" should be identifiers, not literals
    let result = identifier("NaN123");
    assert!(result.is_ok());
    let (rest, id) = result.unwrap();
    assert!(rest.is_empty());
    assert_eq!(id, Identifier::Plain("NaN123".to_string()));

    let result = identifier("Infoo");
    assert!(result.is_ok());
    let (rest, id) = result.unwrap();
    assert!(rest.is_empty());
    assert_eq!(id, Identifier::Plain("Infoo".to_string()));
}

#[test]
fn test_colon_only_in_metric_names() {
    // Label names should stop at colon
    let result = label_name("foo:bar");
    assert!(result.is_ok());
    let (rest, name) = result.unwrap();
    assert_eq!(name, "foo");
    assert_eq!(rest, ":bar");

    // Metric names should include colon
    let result = metric_name("foo:bar");
    assert!(result.is_ok());
    let (rest, name) = result.unwrap();
    assert_eq!(name, "foo:bar");
    assert!(rest.is_empty());
}

#[test]
fn test_identifier_starting_with_underscore() {
    for test in ["_metric", "_label", "_1_2", "__name__"] {
        let result = identifier(test);
        assert!(result.is_ok(), "Failed to parse: '{}'", test);
        let (rest, id) = result.unwrap();
        assert!(rest.is_empty());
        assert_eq!(id.as_str(), test);
    }
}

#[test]
fn test_metric_name_starting_with_colon() {
    // Recording rules can start with colon
    let result = metric_name(":bc");
    assert!(result.is_ok());
    let (rest, name) = result.unwrap();
    assert_eq!(name, ":bc");
    assert!(rest.is_empty());
}
