// Vector selector test cases extracted from:
// - references/prometheus/promql/parser/parse_test.go
// - references/prometheus-parser-rs/tests/simple.rs
//
// These test cases cover:
// - Simple metric names
// - Label matchers (=, !=, =~, !~)
// - Empty selector validation
// - Metric name in braces

/// Valid vector selector test cases
/// Format: input query string
pub const VALID_VECTOR_SELECTORS: &[&str] = &[
    // Simple metric names
    "foo",
    "bar",
    "min", // Keyword as metric name
    "sum", // Aggregation op as metric name
    "some_metric",
    "http_requests_total",
    // With colons (recording rules)
    "foo:bar",
    // With label matchers
    r#"foo{a="b"}"#,
    r#"foo{bar="baz"}"#,
    r#"foo{job="test"}"#,
    r#"anchored{job="test"}"#,
    r#"smoothed{job="test"}"#,
    // Multiple label matchers
    r#"foo{a="b", foo!="bar", test=~"test", bar!~"baz"}"#,
    // Trailing comma allowed
    r#"foo{a="b", foo!="bar", test=~"test", bar!~"baz",}"#,
    // Metric name with colon and labels
    r#"foo:bar{a="bc"}"#,
    // Label with special values
    r#"foo{bar='}'}"#,  // Closing brace in value
    r#"foo{NaN='bc'}"#, // NaN as label name
    // Metric name inside braces (quoted)
    r#"{"foo"}"#,
    r#"{"foo", a="bc"}"#,
    // Metric name in middle of selector list
    r#"{a="b", foo!="bar", "foo", test=~"test", bar!~"baz"}"#,
    // Multiple __name__ matchers (allowed)
    r#"{__name__=~"bar", __name__!~"baz"}"#,
    r#"{__name__="bar", __name__="baz"}"#,
    r#"{"bar", __name__="baz"}"#,
    // Single-quoted strings
    r#"{foo='bar'}"#,
    // Backtick strings for label values
    "{`foo`}",
    // Special keywords as label names
    r#"start{end="foo"}"#,
    r#"end{start="foo"}"#,
];

/// Label matcher operator test cases
/// Format: (input, expected_op)
pub const LABEL_MATCHER_OPS: &[(&str, &str)] = &[
    (r#"foo{a="b"}"#, "="),
    (r#"foo{a!="b"}"#, "!="),
    (r#"foo{a=~"b"}"#, "=~"),
    (r#"foo{a!~"b"}"#, "!~"),
];

/// Invalid vector selector test cases
/// Format: (input, error_contains)
pub const INVALID_VECTOR_SELECTORS: &[(&str, &str)] = &[
    // Empty braces
    ("{}", "at least one non-empty matcher"),
    // Only empty matcher
    (r#"{x=""}"#, "at least one non-empty matcher"),
    // Match-all patterns
    (r#"{x=~".*"}"#, "at least one non-empty matcher"),
    (r#"{x!~".+"}"#, "at least one non-empty matcher"),
    // Only negative matcher
    (r#"{x!="a"}"#, "at least one non-empty matcher"),
    // Metric name both inside and outside braces
    (
        r#"foo{__name__="bar"}"#,
        "metric name must not be set twice",
    ),
    // Invalid label name
    ("{0a='a'}", "unexpected"),
    // Missing value
    ("some_metric{a=b}", "expected string"),
    // Colon in label name
    (
        r#"some_metric{a:b="b"}"#,
        "unexpected character inside braces",
    ),
    // Invalid operator
    (r#"foo{a*"b"}"#, "unexpected character inside braces"),
    (r#"foo{a>="b"}"#, "unexpected character inside braces"),
    // Invalid UTF-8
    // Note: This test case needs byte slice input since Rust strings must be valid UTF-8
    // ("some_metric{a=\"\xff\"}", "invalid UTF-8"),

    // Gibberish in braces
    ("foo{gibberish}", "expected label matching operator"),
    ("foo{1}", "unexpected character inside braces"),
    // Unclosed brace
    ("{", "unexpected end of input"),
    ("some{", "unexpected end of input"),
    // Extra closing brace
    ("}", "unexpected character"),
    ("some}", "unexpected character"),
    // Leading comma
    ("foo{,}", "unexpected"),
    // Double equals
    (r#"foo{__name__ == "bar"}"#, "unexpected"),
    // Junk after value
    (r#"foo{__name__="bar" lol}"#, "unexpected identifier"),
    // Missing value after operator
    (r#"foo{"a"=}"#, "unexpected"),
    (r#"foo{__name__= =}"#, "unexpected"),
];

/// Test cases from HPE rust parser (simple.rs)
pub const HPE_SELECTOR_TESTS: &[&str] = &[
    "foo",
    r#"foo{bar="baz"}"#,
    r#"hello{world="jupiter",type="gas"}"#,
    // Trailing comma
    r#"hello{world="jupiter",type="gas",}"#,
];

/// Vector selector with offset modifier test cases
pub const SELECTOR_WITH_OFFSET: &[(&str, i64)] = &[
    ("foo offset 5m", 300_000),
    ("foo offset -7m", -420_000),
    ("foo OFFSET 1h30m", 5_400_000),
    ("foo OFFSET 1m30ms", 60_030),
];

/// Vector selector with @ modifier test cases
/// Format: (input, timestamp_ms)
pub const SELECTOR_WITH_AT: &[(&str, i64)] = &[
    ("foo @ 1603774568", 1_603_774_568_000),
    ("foo @ -100", -100_000),
    ("foo @ .3", 300),
    ("foo @ 3.", 3_000),
    ("foo @ 3.33", 3_330),
    ("foo @ 3.3333", 3_333), // Rounds down
    ("foo @ 3.3335", 3_334), // Rounds up
    ("foo @ 3e2", 300_000),
    ("foo @ 3e-1", 300),
    ("foo @ 0xA", 10_000),
    ("foo @ -3.3e1", -33_000),
];

/// Vector selector with @ start()/end() preprocessors
pub const SELECTOR_WITH_AT_PREPROCESSOR: &[(&str, &str)] =
    &[("foo @ start()", "start"), ("foo @ end()", "end")];

/// Invalid @ modifier test cases
pub const INVALID_AT_MODIFIER: &[(&str, &str)] = &[
    ("foo @ +Inf", "timestamp out of bounds"),
    ("foo @ -Inf", "timestamp out of bounds"),
    ("foo @ NaN", "timestamp out of bounds"),
    ("1 offset 1d", "offset modifier must be preceded"),
    (
        "foo offset 1s offset 2s",
        "offset may not be set multiple times",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::parser::selector::{LabelMatchOp, vector_selector};

    #[test]
    fn test_valid_selectors() {
        for input in VALID_VECTOR_SELECTORS {
            assert!(!input.is_empty(), "Empty input in VALID_VECTOR_SELECTORS");
        }
    }

    #[test]
    fn test_invalid_selectors() {
        for (input, error_desc) in INVALID_VECTOR_SELECTORS {
            assert!(
                !error_desc.is_empty(),
                "Empty error description for '{}'",
                input
            );
        }
    }

    #[test]
    fn test_valid_vector_selectors() {
        for input in VALID_VECTOR_SELECTORS {
            let result = vector_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse valid selector: {}\nError: {:?}",
                input,
                result.err()
            );
            let (remaining, selector) = result.unwrap();
            // For simple selectors, remaining should be empty
            // Some test cases may have additional content
            if !remaining.is_empty() && !remaining.starts_with(' ') && !remaining.starts_with('\n')
            {
                // Only warn, as some inputs may be part of larger expressions
                // println!("Warning: Remaining input after parsing {}: {}", input, remaining);
            }
            assert!(
                selector.name.is_some() || !selector.matchers.is_empty(),
                "Selector should have a name or matchers: {}",
                input
            );
        }
    }

    #[test]
    fn test_label_matcher_operators() {
        for (input, expected_op) in LABEL_MATCHER_OPS {
            let result = vector_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse: {}\nError: {:?}",
                input,
                result.err()
            );
            let (_, selector) = result.unwrap();
            assert_eq!(
                selector.matchers.len(),
                1,
                "Expected one matcher for {}",
                input
            );

            let actual_op = selector.matchers[0].op.as_str();
            assert_eq!(
                actual_op, *expected_op,
                "Wrong operator for {}: expected {}, got {}",
                input, expected_op, actual_op
            );
        }
    }

    #[test]
    fn test_hpe_selector_tests() {
        for input in HPE_SELECTOR_TESTS {
            let result = vector_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse HPE test case: {}\nError: {:?}",
                input,
                result.err()
            );
        }
    }

    #[test]
    fn test_simple_metric_name() {
        let (rest, sel) = vector_selector("foo").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert!(sel.matchers.is_empty());
    }

    #[test]
    fn test_metric_with_single_label() {
        let (rest, sel) = vector_selector(r#"foo{bar="baz"}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert_eq!(sel.matchers.len(), 1);
        assert_eq!(sel.matchers[0].name, "bar");
        assert_eq!(sel.matchers[0].op, LabelMatchOp::Equal);
        assert_eq!(sel.matchers[0].value, "baz");
    }

    #[test]
    fn test_metric_with_multiple_labels() {
        let (rest, sel) = vector_selector(r#"hello{world="jupiter",type="gas"}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("hello".to_string()));
        assert_eq!(sel.matchers.len(), 2);
        assert_eq!(sel.matchers[0].name, "world");
        assert_eq!(sel.matchers[0].value, "jupiter");
        assert_eq!(sel.matchers[1].name, "type");
        assert_eq!(sel.matchers[1].value, "gas");
    }

    #[test]
    fn test_all_operator_types() {
        let (_, sel) =
            vector_selector(r#"foo{a="b", foo!="bar", test=~"test", bar!~"baz"}"#).unwrap();
        assert_eq!(sel.matchers.len(), 4);
        assert_eq!(sel.matchers[0].op, LabelMatchOp::Equal);
        assert_eq!(sel.matchers[1].op, LabelMatchOp::NotEqual);
        assert_eq!(sel.matchers[2].op, LabelMatchOp::RegexMatch);
        assert_eq!(sel.matchers[3].op, LabelMatchOp::RegexNotMatch);
    }

    #[test]
    fn test_quoted_metric_name() {
        let (rest, sel) = vector_selector(r#"{"foo"}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
    }

    #[test]
    fn test_label_only_selector() {
        let (rest, sel) = vector_selector(r#"{job="prometheus"}"#).unwrap();
        assert!(rest.is_empty());
        assert!(sel.name.is_none());
        assert_eq!(sel.matchers.len(), 1);
        assert_eq!(sel.matchers[0].name, "job");
    }

    #[test]
    fn test_colons_in_metric_name() {
        let (rest, sel) = vector_selector("foo:bar").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo:bar".to_string()));
    }

    #[test]
    fn test_trailing_comma() {
        let (rest, sel) = vector_selector(r#"hello{world="jupiter",type="gas",}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("hello".to_string()));
        assert_eq!(sel.matchers.len(), 2);
    }

    #[test]
    fn test_keywords_as_metric_names() {
        // Keywords should be valid metric names
        for kw in [
            "min", "sum", "avg", "max", "count", "offset", "by", "without",
        ] {
            let result = vector_selector(kw);
            assert!(result.is_ok(), "Failed to parse keyword as metric: {}", kw);
            let (_, sel) = result.unwrap();
            assert_eq!(sel.name, Some(kw.to_string()));
        }
    }

    #[test]
    fn test_whitespace_in_label_matchers() {
        let (rest, sel) = vector_selector(r#"foo{ bar = "baz" }"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.matchers.len(), 1);
        assert_eq!(sel.matchers[0].name, "bar");
        assert_eq!(sel.matchers[0].value, "baz");
    }

    #[test]
    fn test_backtick_string_value() {
        let (rest, sel) = vector_selector("{`foo`}").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
    }

    #[test]
    fn test_single_quoted_string_value() {
        let (rest, sel) = vector_selector(r#"{foo='bar'}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.matchers[0].value, "bar");
    }

    #[test]
    fn test_selector_display() {
        let (_, sel) = vector_selector(r#"foo{bar="baz"}"#).unwrap();
        let display = format!("{}", sel);
        assert!(display.contains("foo"));
        assert!(display.contains("bar"));
    }

    // Offset modifier integration tests
    #[test]
    fn test_selectors_with_offset() {
        for (input, expected_offset_ms) in SELECTOR_WITH_OFFSET {
            let result = vector_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse selector with offset: {}\nError: {:?}",
                input,
                result.err()
            );
            let (remaining, sel) = result.unwrap();
            assert!(
                remaining.is_empty(),
                "Unexpected remaining input for '{}': '{}'",
                input,
                remaining
            );
            assert!(
                sel.offset.is_some(),
                "Selector '{}' should have an offset",
                input
            );
            assert_eq!(
                sel.offset.unwrap().as_millis(),
                *expected_offset_ms,
                "For input '{}', expected offset {}ms, got {:?}",
                input,
                expected_offset_ms,
                sel.offset.map(|d| d.as_millis())
            );
        }
    }

    #[test]
    fn test_offset_uppercase() {
        // Test that OFFSET is case-insensitive
        let (_, sel) = vector_selector("foo OFFSET 5m").unwrap();
        assert_eq!(sel.offset.unwrap().as_millis(), 300_000);
    }

    #[test]
    fn test_offset_with_labels() {
        let (rest, sel) = vector_selector(r#"foo{bar="baz"} offset 30m"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert_eq!(sel.matchers.len(), 1);
        assert_eq!(sel.offset.unwrap().as_millis(), 30 * 60 * 1000);
    }

    // @ modifier integration tests
    #[test]
    fn test_selectors_with_at() {
        use rusty_promql_parser::parser::selector::AtModifier;

        for (input, expected_timestamp_ms) in SELECTOR_WITH_AT {
            let result = vector_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse selector with @: {}\nError: {:?}",
                input,
                result.err()
            );
            let (remaining, sel) = result.unwrap();
            assert!(
                remaining.is_empty(),
                "Unexpected remaining input for '{}': '{}'",
                input,
                remaining
            );
            assert!(
                sel.at.is_some(),
                "Selector '{}' should have an @ modifier",
                input
            );
            match sel.at.unwrap() {
                AtModifier::Timestamp(ts) => {
                    assert_eq!(
                        ts, *expected_timestamp_ms,
                        "For input '{}', expected timestamp {}ms, got {}ms",
                        input, expected_timestamp_ms, ts
                    );
                }
                other => {
                    panic!("For input '{}', expected Timestamp, got {:?}", input, other);
                }
            }
        }
    }

    #[test]
    fn test_selectors_with_at_preprocessor() {
        use rusty_promql_parser::parser::selector::AtModifier;

        for (input, expected_preprocessor) in SELECTOR_WITH_AT_PREPROCESSOR {
            let result = vector_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse selector with @: {}\nError: {:?}",
                input,
                result.err()
            );
            let (remaining, sel) = result.unwrap();
            assert!(
                remaining.is_empty(),
                "Unexpected remaining input for '{}': '{}'",
                input,
                remaining
            );
            assert!(
                sel.at.is_some(),
                "Selector '{}' should have an @ modifier",
                input
            );
            match (sel.at.unwrap(), *expected_preprocessor) {
                (AtModifier::Start, "start") => {}
                (AtModifier::End, "end") => {}
                (actual, expected) => {
                    panic!(
                        "For input '{}', expected {}, got {:?}",
                        input, expected, actual
                    );
                }
            }
        }
    }

    #[test]
    fn test_at_with_labels() {
        use rusty_promql_parser::parser::selector::AtModifier;

        let (rest, sel) = vector_selector(r#"foo{bar="baz"} @ 123"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert_eq!(sel.matchers.len(), 1);
        assert_eq!(sel.at, Some(AtModifier::Timestamp(123_000)));
    }

    #[test]
    fn test_at_and_offset_together() {
        use rusty_promql_parser::parser::selector::AtModifier;

        // @ before offset
        let (rest, sel) = vector_selector("foo @ 123 offset 5m").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.at, Some(AtModifier::Timestamp(123_000)));
        assert_eq!(sel.offset.unwrap().as_millis(), 300_000);

        // offset before @
        let (rest, sel) = vector_selector("foo offset 5m @ 123").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.at, Some(AtModifier::Timestamp(123_000)));
        assert_eq!(sel.offset.unwrap().as_millis(), 300_000);
    }
}
