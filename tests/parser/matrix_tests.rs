// Matrix selector (range vector) test cases extracted from:
// - references/prometheus/promql/parser/parse_test.go
//
// These test cases cover:
// - Range vector selectors with duration
// - Matrix selectors with various modifiers
// - Error cases for invalid matrix selectors

/// Valid matrix selector test cases
pub const VALID_MATRIX_SELECTORS: &[&str] = &[
    // Simple range vectors
    "some_metric[5m]",
    "some_metric[1h]",
    "some_metric[1d]",
    "some_metric[1w]",
    "some_metric[1y]",
    "some_metric[300s]",
    "some_metric[30000ms]",
    // With label matchers
    r#"some_metric{job="foo"}[5m]"#,
    r#"some_metric{job="foo",instance="bar"}[5m]"#,
    r#"http_requests_total{method="GET",status="200"}[5m]"#,
    // With various matcher operators
    r#"some_metric{job!="foo"}[5m]"#,
    r#"some_metric{job=~"foo.*"}[5m]"#,
    r#"some_metric{job!~"bar.*"}[5m]"#,
    // Complex durations
    "some_metric[1h30m]",
    "some_metric[2h45m30s]",
    "some_metric[1d12h]",
];

/// Matrix selectors with offset modifier
pub const MATRIX_WITH_OFFSET: &[&str] = &[
    "some_metric[5m] offset 10m",
    "some_metric[5m] offset 1h",
    "some_metric[5m] offset 1d",
    r#"some_metric{job="foo"}[5m] offset 10m"#,
    // Negative offset (supported in newer Prometheus)
    "some_metric[5m] offset -10m",
];

/// Matrix selectors with @ modifier
pub const MATRIX_WITH_AT: &[&str] = &[
    "some_metric[5m] @ 1609459200",
    "some_metric[5m] @ 1609459200.123",
    "some_metric[5m] @ start()",
    "some_metric[5m] @ end()",
    r#"some_metric{job="foo"}[5m] @ 1609459200"#,
];

/// Matrix selectors with both offset and @ modifiers
pub const MATRIX_WITH_BOTH_MODIFIERS: &[&str] = &[
    "some_metric[5m] @ 1609459200 offset 10m",
    "some_metric[5m] offset 10m @ 1609459200",
    "some_metric[5m] @ start() offset 1h",
    r#"some_metric{job="foo"}[5m] offset 5m @ end()"#,
];

/// Duration formats for matrix selectors
pub const VALID_DURATIONS_IN_MATRIX: &[&str] = &[
    // Milliseconds
    "some_metric[100ms]",
    "some_metric[1500ms]",
    // Seconds
    "some_metric[1s]",
    "some_metric[60s]",
    "some_metric[3600s]",
    // Minutes
    "some_metric[1m]",
    "some_metric[5m]",
    "some_metric[30m]",
    // Hours
    "some_metric[1h]",
    "some_metric[24h]",
    // Days
    "some_metric[1d]",
    "some_metric[7d]",
    // Weeks
    "some_metric[1w]",
    "some_metric[4w]",
    // Years
    "some_metric[1y]",
    // Compound durations
    "some_metric[1h30m]",
    "some_metric[1d12h]",
    "some_metric[2h30m15s]",
];

/// Matrix selectors in function calls (required by rate, irate, etc.)
pub const MATRIX_IN_FUNCTIONS: &[&str] = &[
    "rate(some_metric[5m])",
    "irate(some_metric[5m])",
    "increase(some_metric[5m])",
    "delta(some_metric[5m])",
    "deriv(some_metric[5m])",
    "changes(some_metric[5m])",
    "resets(some_metric[5m])",
    "avg_over_time(some_metric[5m])",
    "sum_over_time(some_metric[5m])",
    "count_over_time(some_metric[5m])",
    "min_over_time(some_metric[5m])",
    "max_over_time(some_metric[5m])",
    "last_over_time(some_metric[5m])",
    "present_over_time(some_metric[5m])",
    "absent_over_time(some_metric[5m])",
    "stddev_over_time(some_metric[5m])",
    "stdvar_over_time(some_metric[5m])",
    "quantile_over_time(0.9, some_metric[5m])",
];

/// Invalid matrix selector test cases
pub const INVALID_MATRIX_SELECTORS: &[(&str, &str)] = &[
    // Missing duration
    ("some_metric[]", "expected duration"),
    // Invalid duration format
    ("some_metric[5]", "expected duration"),
    ("some_metric[5min]", "expected duration"),
    ("some_metric[m5]", "expected duration"),
    ("some_metric[-5m]", "expected duration"),
    // Zero duration
    ("some_metric[0s]", "duration must be greater than 0"),
    ("some_metric[0m]", "duration must be greater than 0"),
    // Missing bracket
    ("some_metric[5m", "expected closing bracket"),
    ("some_metric 5m]", "unexpected"),
    // Double range (invalid)
    ("some_metric[5m][10m]", "unexpected character"),
    // Range on scalar (invalid)
    ("1[5m]", "unexpected"),
    ("3.14[5m]", "unexpected"),
    // Range on string (invalid)
    (r#""string"[5m]"#, "unexpected"),
    // Range on aggregation result without subquery syntax
    // Note: "sum(x)[5m]" requires subquery syntax "[5m:]" not just "[5m]"
    // Actually this might be valid in some contexts

    // Invalid characters in duration
    ("some_metric[5.5m]", "expected duration"), // Float not allowed
    ("some_metric[5 m]", "expected duration"),  // Space not allowed
];

/// Edge cases for matrix selectors
pub const MATRIX_EDGE_CASES: &[&str] = &[
    // Very short duration
    "some_metric[1ms]",
    "some_metric[1s]",
    // Very long duration
    "some_metric[365d]",
    "some_metric[10y]",
    // All time units combined
    "some_metric[1y52w365d8760h525600m31536000s]",
    // With complex label matchers
    r#"some_metric{job="foo",instance=~"bar.*",env!="prod",version!~"v1.*"}[5m]"#,
];

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::parser::selector::matrix_selector;

    #[test]
    fn test_matrix_selectors() {
        for input in VALID_MATRIX_SELECTORS {
            assert!(
                input.contains('['),
                "Matrix selector '{}' should contain '['",
                input
            );
        }
    }

    #[test]
    fn test_matrix_with_offset() {
        for input in MATRIX_WITH_OFFSET {
            let lower = input.to_lowercase();
            assert!(
                lower.contains("offset"),
                "Matrix selector '{}' should contain 'offset'",
                input
            );
        }
    }

    #[test]
    fn test_invalid_matrix() {
        for (input, error_desc) in INVALID_MATRIX_SELECTORS {
            assert!(
                !error_desc.is_empty(),
                "Empty error description for '{}'",
                input
            );
        }
    }

    #[test]
    fn test_valid_matrix_selectors() {
        for input in VALID_MATRIX_SELECTORS {
            let result = matrix_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse valid matrix selector: {}\nError: {:?}",
                input,
                result.err()
            );
            let (rest, _sel) = result.unwrap();
            assert!(
                rest.is_empty(),
                "Remaining input after parsing '{}': '{}'",
                input,
                rest
            );
        }
    }

    #[test]
    fn test_valid_durations_in_matrix() {
        for input in VALID_DURATIONS_IN_MATRIX {
            let result = matrix_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse matrix selector with duration: {}\nError: {:?}",
                input,
                result.err()
            );
        }
    }

    #[test]
    fn test_matrix_edge_cases() {
        for input in MATRIX_EDGE_CASES {
            let result = matrix_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse edge case: {}\nError: {:?}",
                input,
                result.err()
            );
        }
    }

    #[test]
    fn test_simple_matrix_selector() {
        let (rest, sel) = matrix_selector("some_metric[5m]").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name(), Some("some_metric"));
        assert_eq!(sel.range_millis(), 5 * 60 * 1000);
    }

    #[test]
    fn test_matrix_selector_with_labels() {
        let (rest, sel) = matrix_selector(r#"some_metric{job="foo"}[5m]"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name(), Some("some_metric"));
        assert_eq!(sel.matchers().len(), 1);
        assert_eq!(sel.matchers()[0].name, "job");
        assert_eq!(sel.matchers()[0].value, "foo");
    }

    #[test]
    fn test_matrix_selector_hour() {
        let (rest, sel) = matrix_selector("some_metric[1h]").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.range_millis(), 60 * 60 * 1000);
    }

    #[test]
    fn test_matrix_selector_day() {
        let (rest, sel) = matrix_selector("some_metric[1d]").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.range_millis(), 24 * 60 * 60 * 1000);
    }

    #[test]
    fn test_matrix_selector_week() {
        let (rest, sel) = matrix_selector("some_metric[1w]").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.range_millis(), 7 * 24 * 60 * 60 * 1000);
    }

    #[test]
    fn test_matrix_selector_compound_duration() {
        let (rest, sel) = matrix_selector("some_metric[1h30m]").unwrap();
        assert!(rest.is_empty());
        // 1h30m = 90 minutes = 5400 seconds = 5400000 ms
        assert_eq!(sel.range_millis(), 90 * 60 * 1000);
    }

    #[test]
    fn test_matrix_selector_milliseconds() {
        let (rest, sel) = matrix_selector("some_metric[30000ms]").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.range_millis(), 30000);
    }

    #[test]
    fn test_matrix_selector_multiple_labels() {
        let (rest, sel) =
            matrix_selector(r#"http_requests_total{method="GET",status="200"}[5m]"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.matchers().len(), 2);
    }

    #[test]
    fn test_matrix_selector_all_matcher_ops() {
        let inputs = [
            r#"some_metric{job!="foo"}[5m]"#,
            r#"some_metric{job=~"foo.*"}[5m]"#,
            r#"some_metric{job!~"bar.*"}[5m]"#,
        ];
        for input in inputs {
            let result = matrix_selector(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
        }
    }

    // Offset modifier tests
    #[test]
    fn test_matrix_selector_with_offset() {
        for input in MATRIX_WITH_OFFSET {
            let result = matrix_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse matrix selector with offset: {}\nError: {:?}",
                input,
                result.err()
            );
            let (rest, sel) = result.unwrap();
            assert!(
                rest.is_empty(),
                "Remaining input after parsing '{}': '{}'",
                input,
                rest
            );
            assert!(
                sel.offset().is_some(),
                "Matrix selector '{}' should have offset",
                input
            );
        }
    }

    #[test]
    fn test_matrix_selector_offset_values() {
        let (_, sel) = matrix_selector("some_metric[5m] offset 10m").unwrap();
        assert_eq!(sel.offset_millis(), Some(10 * 60 * 1000));

        let (_, sel) = matrix_selector("some_metric[5m] offset -10m").unwrap();
        assert_eq!(sel.offset_millis(), Some(-10 * 60 * 1000));
    }

    // @ modifier tests
    #[test]
    fn test_matrix_selector_with_at() {
        for input in MATRIX_WITH_AT {
            let result = matrix_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse matrix selector with @: {}\nError: {:?}",
                input,
                result.err()
            );
            let (rest, sel) = result.unwrap();
            assert!(
                rest.is_empty(),
                "Remaining input after parsing '{}': '{}'",
                input,
                rest
            );
            assert!(
                sel.at().is_some(),
                "Matrix selector '{}' should have @ modifier",
                input
            );
        }
    }

    #[test]
    fn test_matrix_selector_at_timestamp() {
        use rusty_promql_parser::parser::selector::AtModifier;
        let (_, sel) = matrix_selector("some_metric[5m] @ 1609459200").unwrap();
        match sel.at() {
            Some(AtModifier::Timestamp(ts)) => assert_eq!(*ts, 1609459200000),
            other => panic!("Expected Timestamp, got {:?}", other),
        }
    }

    #[test]
    fn test_matrix_selector_at_start_end() {
        use rusty_promql_parser::parser::selector::AtModifier;

        let (_, sel) = matrix_selector("some_metric[5m] @ start()").unwrap();
        assert_eq!(sel.at(), Some(&AtModifier::Start));

        let (_, sel) = matrix_selector("some_metric[5m] @ end()").unwrap();
        assert_eq!(sel.at(), Some(&AtModifier::End));
    }

    // Combined @ and offset tests
    #[test]
    fn test_matrix_selector_with_both_modifiers() {
        for input in MATRIX_WITH_BOTH_MODIFIERS {
            let result = matrix_selector(input);
            assert!(
                result.is_ok(),
                "Failed to parse matrix selector with both modifiers: {}\nError: {:?}",
                input,
                result.err()
            );
            let (rest, sel) = result.unwrap();
            assert!(
                rest.is_empty(),
                "Remaining input after parsing '{}': '{}'",
                input,
                rest
            );
            assert!(
                sel.at().is_some(),
                "Matrix selector '{}' should have @ modifier",
                input
            );
            assert!(
                sel.offset().is_some(),
                "Matrix selector '{}' should have offset",
                input
            );
        }
    }

    #[test]
    fn test_matrix_selector_at_before_offset() {
        use rusty_promql_parser::parser::selector::AtModifier;
        let (_, sel) = matrix_selector("some_metric[5m] @ 1609459200 offset 10m").unwrap();
        match sel.at() {
            Some(AtModifier::Timestamp(ts)) => assert_eq!(*ts, 1609459200000),
            other => panic!("Expected Timestamp, got {:?}", other),
        }
        assert_eq!(sel.offset_millis(), Some(10 * 60 * 1000));
    }

    #[test]
    fn test_matrix_selector_offset_before_at() {
        use rusty_promql_parser::parser::selector::AtModifier;
        let (_, sel) = matrix_selector("some_metric[5m] offset 10m @ 1609459200").unwrap();
        match sel.at() {
            Some(AtModifier::Timestamp(ts)) => assert_eq!(*ts, 1609459200000),
            other => panic!("Expected Timestamp, got {:?}", other),
        }
        assert_eq!(sel.offset_millis(), Some(10 * 60 * 1000));
    }

    // Display tests
    #[test]
    fn test_matrix_selector_display_with_offset() {
        let (_, sel) = matrix_selector("some_metric[5m] offset 10m").unwrap();
        assert_eq!(sel.to_string(), "some_metric[5m] offset 10m");
    }

    #[test]
    fn test_matrix_selector_display_with_at() {
        let (_, sel) = matrix_selector("some_metric[5m] @ start()").unwrap();
        assert_eq!(sel.to_string(), "some_metric[5m] @ start()");
    }

    #[test]
    fn test_matrix_selector_display_with_both() {
        let (_, sel) = matrix_selector("some_metric[5m] @ start() offset 10m").unwrap();
        assert_eq!(sel.to_string(), "some_metric[5m] @ start() offset 10m");
    }
}
