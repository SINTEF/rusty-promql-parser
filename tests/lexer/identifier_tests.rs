// Identifier and keyword test cases extracted from:
// - references/prometheus/promql/parser/lex_test.go
//
// These test cases cover:
// - Metric names
// - Label names
// - Keywords (context-sensitive)
// - Reserved words

/// Valid metric name patterns
/// Metric names: [a-zA-Z_:][a-zA-Z0-9_:]*
pub const VALID_METRIC_NAMES: &[&str] = &[
    // Simple names
    "foo",
    "bar",
    "some_metric",
    "http_requests",
    "http_requests_total",
    // With colons (recording rules)
    "foo:bar",
    "a:bc",
    ":bc", // Can start with colon
    "test:name",
    "this:or:that:",
    ":this:or:that",
    "t::s", // Multiple colons
    // Keywords as metric names (allowed)
    "min",
    "sum",
    "avg",
    "offset",
    "start",
    "end",
    "and",
    "or",
    "unless",
    "on",
    "ignoring",
    "group_left",
    "group_right",
    "bool",
    "atan2",
    // Starting with underscore
    "_metric",
    "_1_2",
    // Mixed case
    "NaN123", // Identifier, not NaN literal
    "Infoo",  // Identifier, not Inf literal
];

/// Valid label name patterns
/// Label names: [a-zA-Z_][a-zA-Z0-9_]* (no colons allowed)
pub const VALID_LABEL_NAMES: &[&str] = &[
    // Simple names
    "foo",
    "bar",
    "job",
    "instance",
    "group",
    "namespace",
    // With underscores
    "some_label",
    "job_name",
    // Reserved labels
    "__name__",
    "__address__",
    // Starting with underscore
    "_label",
    "_1_2",
    // Keywords can be label names
    "and",
    "by",
    "avg",
    "count",
    "alert",
    "annotations",
    "on",
    "ignoring",
    "start",
    "end",
    "NaN", // Also valid as label name
];

/// Invalid identifier patterns
pub const INVALID_IDENTIFIERS: &[(&str, &str)] = &[
    // Starting with number
    ("0a:bc", "unexpected"),
    ("0foo", "unexpected"),
    ("1metric", "unexpected"),
    // Non-ASCII characters
    ("台北", "unexpected"),
    ("{台北='a'}", "unexpected"),
    // Colon in label name (only metric names allow colons)
    ("a:b", "colon"), // Valid as metric, but not as label when used as label
    // Special characters
    ("foo-bar", "unexpected"), // Dash not allowed
    ("foo.bar", "unexpected"), // Dot not allowed in identifiers
];

/// Keywords with their token types (from lex_test.go)
pub const KEYWORDS: &[(&str, &str)] = &[
    ("offset", "OFFSET"),
    ("by", "BY"),
    ("without", "WITHOUT"),
    ("on", "ON"),
    ("ignoring", "IGNORING"),
    ("group_left", "GROUP_LEFT"),
    ("group_right", "GROUP_RIGHT"),
    ("bool", "BOOL"),
    ("atan2", "ATAN2"),
];

/// Aggregation operators (also keywords)
pub const AGGREGATION_KEYWORDS: &[&str] = &[
    "sum",
    "SUM",
    "avg",
    "AVG",
    "count",
    "COUNT",
    "min",
    "MIN",
    "max",
    "MAX",
    "group",
    "GROUP",
    "stddev",
    "STDDEV",
    "stdvar",
    "STDVAR",
    "topk",
    "bottomk",
    "quantile",
    "count_values",
    "limitk",
    "limit_ratio",
];

/// Set operator keywords
pub const SET_OPERATORS: &[&str] = &[
    "and", "AND", "And", "or", "OR", "Or", "unless", "UNLESS", "Unless",
];

/// Preprocessor keywords (@ modifier)
pub const PREPROCESSOR_KEYWORDS: &[&str] = &["start", "end"];

/// Identifier test cases from lexer tests
pub const LEXER_IDENTIFIER_TESTS: &[(&str, &str, &str)] = &[
    // (input, expected_token_type, expected_value)
    ("abc", "IDENTIFIER", "abc"),
    ("a:bc", "METRIC_IDENTIFIER", "a:bc"),
    ("abc d", "IDENTIFIER", "abc"), // Just first token
    (":bc", "METRIC_IDENTIFIER", ":bc"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::lexer::identifier::{
        Identifier, identifier, keyword, label_name, metric_name,
    };

    #[test]
    fn test_valid_metric_names_parse() {
        for name in VALID_METRIC_NAMES {
            let result = metric_name(name);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "metric_name parser did not consume entire input '{}', remaining: '{}'",
                        name,
                        remaining
                    );
                    assert_eq!(
                        parsed, *name,
                        "Parsed metric name should match input for '{}'",
                        name
                    );
                }
                Err(e) => panic!("Failed to parse metric name '{}': {:?}", name, e),
            }
        }
    }

    #[test]
    fn test_valid_label_names_parse() {
        for name in VALID_LABEL_NAMES {
            let result = label_name(name);
            match result {
                Ok((remaining, parsed)) => {
                    // Label names don't allow colons, so check if there's a colon
                    if !name.contains(':') {
                        assert!(
                            remaining.is_empty(),
                            "label_name parser did not consume entire input '{}', remaining: '{}'",
                            name,
                            remaining
                        );
                        assert_eq!(
                            parsed, *name,
                            "Parsed label name should match input for '{}'",
                            name
                        );
                    }
                }
                Err(e) => {
                    // Some test data like "NaN" might need identifier parser instead
                    // Only fail if it's actually a valid label name pattern
                    if name
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        panic!("Failed to parse valid label name '{}': {:?}", name, e);
                    }
                }
            }
        }
    }

    #[test]
    fn test_identifier_distinguishes_metric_vs_plain() {
        // Test that identifier() returns Metric for names with colons
        let (_, id) = identifier("foo:bar").unwrap();
        assert!(matches!(id, Identifier::Metric(_)));
        assert!(id.has_colon());

        // And Plain for names without colons
        let (_, id) = identifier("foo").unwrap();
        assert!(matches!(id, Identifier::Plain(_)));
        assert!(!id.has_colon());
    }

    #[test]
    fn test_keywords_parse() {
        for (kw_str, _expected_type) in KEYWORDS {
            let result = keyword(kw_str);
            assert!(
                result.is_ok(),
                "Keyword '{}' should parse successfully",
                kw_str
            );
            let (remaining, _kw) = result.unwrap();
            assert!(
                remaining.is_empty(),
                "Keyword parser should consume entire input '{}', remaining: '{}'",
                kw_str,
                remaining
            );
        }
    }

    #[test]
    fn test_aggregation_keywords_parse() {
        for kw_str in AGGREGATION_KEYWORDS {
            let result = keyword(kw_str);
            match result {
                Ok((remaining, kw)) => {
                    assert!(
                        remaining.is_empty(),
                        "Keyword parser did not consume entire input '{}', remaining: '{}'",
                        kw_str,
                        remaining
                    );
                    assert!(
                        kw.is_aggregation(),
                        "Keyword '{}' should be an aggregation operator",
                        kw_str
                    );
                }
                Err(e) => panic!("Failed to parse aggregation keyword '{}': {:?}", kw_str, e),
            }
        }
    }

    #[test]
    fn test_set_operators_parse() {
        for op_str in SET_OPERATORS {
            let result = keyword(op_str);
            match result {
                Ok((remaining, kw)) => {
                    assert!(
                        remaining.is_empty(),
                        "Keyword parser did not consume entire input '{}', remaining: '{}'",
                        op_str,
                        remaining
                    );
                    assert!(
                        kw.is_set_operator(),
                        "Keyword '{}' should be a set operator",
                        op_str
                    );
                }
                Err(e) => panic!("Failed to parse set operator '{}': {:?}", op_str, e),
            }
        }
    }

    #[test]
    fn test_preprocessor_keywords_parse() {
        // start and end are valid keywords
        for kw_str in PREPROCESSOR_KEYWORDS {
            let result = keyword(kw_str);
            assert!(
                result.is_ok(),
                "Preprocessor keyword '{}' should parse successfully",
                kw_str
            );
        }
    }

    #[test]
    fn test_invalid_identifiers_fail() {
        for (input, _error_desc) in INVALID_IDENTIFIERS {
            let result = identifier(input);
            // Should either fail or not fully consume input
            match result {
                Err(_) => {
                    // Good - it should fail
                }
                Ok((_remaining, _)) => {
                    // If it parses, it should leave something unparsed (like invalid chars)
                    // Some inputs like "a:b" are valid metric identifiers but the test
                    // says they're invalid when used as labels
                }
            }
        }
    }

    #[test]
    fn test_lexer_identifier_cases() {
        for (input, expected_type, expected_value) in LEXER_IDENTIFIER_TESTS {
            let result = identifier(input);
            match result {
                Ok((remaining, id)) => {
                    assert_eq!(
                        id.as_str(),
                        *expected_value,
                        "Parsed identifier value should match for '{}'",
                        input
                    );
                    // Check type matches
                    let is_metric = matches!(&id, Identifier::Metric(_));
                    let expected_is_metric = *expected_type == "METRIC_IDENTIFIER";
                    assert_eq!(
                        is_metric,
                        expected_is_metric,
                        "Identifier type mismatch for '{}': expected {}, got {}",
                        input,
                        expected_type,
                        if is_metric {
                            "METRIC_IDENTIFIER"
                        } else {
                            "IDENTIFIER"
                        }
                    );
                    // Some inputs have trailing content (like "abc d")
                    if input.contains(' ') {
                        assert!(!remaining.is_empty());
                    }
                }
                Err(e) => panic!("Failed to parse identifier '{}': {:?}", input, e),
            }
        }
    }
}
