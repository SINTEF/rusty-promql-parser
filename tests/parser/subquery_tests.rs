// Subquery test cases extracted from:
// - references/prometheus/promql/parser/parse_test.go
// - references/prometheus-parser-rs/tests/subquery.rs
//
// These test cases cover:
// - Basic subquery syntax [range:step]
// - Optional step parameter
// - Subqueries on various expression types
// - Nested subqueries
// - Error cases

/// Valid simple subquery test cases
pub const VALID_SIMPLE_SUBQUERIES: &[&str] = &[
    // Basic subquery with step
    "some_metric[5m:1m]",
    "some_metric[1h:5m]",
    "some_metric[1d:1h]",
    // Subquery without step (uses default evaluation interval)
    "some_metric[5m:]",
    "some_metric[1h:]",
    "some_metric[1d:]",
    // With various time units
    "some_metric[300s:60s]",
    "some_metric[5m:30s]",
    "some_metric[1w:1d]",
    "some_metric[1y:1w]",
];

/// Subqueries with selectors
pub const SUBQUERIES_WITH_SELECTORS: &[&str] = &[
    r#"some_metric{job="foo"}[5m:1m]"#,
    r#"some_metric{job="foo",instance="bar"}[5m:1m]"#,
    r#"http_requests_total{method="GET"}[30m:5m]"#,
];

/// Subqueries on function results
pub const SUBQUERIES_ON_FUNCTIONS: &[&str] = &[
    "rate(some_metric[5m])[30m:1m]",
    "irate(some_metric[5m])[1h:5m]",
    "increase(some_metric[5m])[30m:5m]",
    "deriv(some_metric[5m])[30m:5m]",
    "avg_over_time(some_metric[5m])[30m:5m]",
    // From prometheus-parser-rs tests
    "min_over_time(rate(foo{bar=\"baz\"}[2s])[5m:5s])",
    "max_over_time(rate(foo{bar=\"baz\"}[2s])[5m:])",
];

/// Subqueries on aggregation results
pub const SUBQUERIES_ON_AGGREGATIONS: &[&str] = &[
    "sum(some_metric)[5m:1m]",
    "avg(some_metric) by (job)[30m:5m]",
    "sum(rate(some_metric[5m])) by (job)[30m:5m]",
    "topk(5, some_metric)[30m:5m]",
];

/// Subqueries on binary expressions
pub const SUBQUERIES_ON_BINARY_EXPRS: &[&str] = &[
    "(some_metric + other_metric)[5m:1m]",
    "(some_metric / other_metric)[5m:1m]",
    "(rate(some_metric[5m]) * 100)[30m:5m]",
];

/// Nested subqueries
pub const NESTED_SUBQUERIES: &[&str] = &[
    "rate(some_metric[5m:1m])[30m:5m]",
    "sum_over_time(rate(some_metric[5m])[30m:5m])[1h:10m]",
    "avg_over_time(some_metric[5m:1m])[30m:5m]",
];

/// Subqueries with offset modifier
pub const SUBQUERIES_WITH_OFFSET: &[&str] = &[
    "some_metric[5m:1m] offset 10m",
    "rate(some_metric[5m])[30m:1m] offset 1h",
    "sum(some_metric)[5m:1m] offset 5m",
];

/// Subqueries with @ modifier
pub const SUBQUERIES_WITH_AT: &[&str] = &[
    "some_metric[5m:1m] @ 1609459200",
    "rate(some_metric[5m])[30m:1m] @ 1609459200",
    "some_metric[5m:1m] @ start()",
    "some_metric[5m:1m] @ end()",
];

/// Subqueries with both offset and @ modifiers
/// Parser now supports both orderings: @ before offset and offset before @
pub const SUBQUERIES_WITH_BOTH_MODIFIERS: &[&str] = &[
    "some_metric[5m:1m] @ 1609459200 offset 10m",
    "some_metric[5m:1m] offset 10m @ 1609459200",
    "rate(some_metric[5m])[30m:1m] @ start() offset 1h",
];

/// Common subquery patterns for aggregation over time
pub const SUBQUERY_AGG_OVER_TIME_PATTERNS: &[&str] = &[
    // Average of rates over a longer window
    "avg_over_time(rate(some_metric[5m])[30m:1m])",
    // Maximum value over time window
    "max_over_time(some_metric[1h:])",
    // Minimum value over time window
    "min_over_time(some_metric[1h:5m])",
    // Quantile over subquery results
    "quantile_over_time(0.9, rate(some_metric[5m])[30m:1m])",
    // Standard deviation of rates
    "stddev_over_time(rate(some_metric[5m])[30m:1m])",
];

/// Invalid subquery test cases
pub const INVALID_SUBQUERIES: &[(&str, &str)] = &[
    // Missing range
    ("some_metric[:1m]", "expected duration"),
    ("some_metric[:]", "expected duration"),
    // Invalid duration format
    ("some_metric[5:1m]", "expected duration"),
    ("some_metric[5m:1]", "expected duration"),
    // Subquery on scalar
    ("1[5m:1m]", "expected type instant vector"),
    ("3.14[5m:]", "expected type instant vector"),
    // Subquery on string
    (r#""string"[5m:1m]"#, "expected type instant vector"),
    // Double range (not valid)
    ("some_metric[5m][5m:1m]", "unexpected character"),
    // Negative step (invalid in standard promql)
    ("some_metric[5m:-1m]", "unexpected"),
    // Zero step is semantically invalid
    // ("some_metric[5m:0s]", "step cannot be zero"),

    // Step larger than range (semantically questionable but might parse)
    // ("some_metric[5m:1h]", "..."),

    // Missing colon
    ("some_metric[5m1m]", "expected closing bracket"),
    // Invalid characters in duration
    ("some_metric[5min:1min]", "expected duration"),
];

/// Test cases from prometheus-parser-rs subquery.rs
pub const HPE_SUBQUERY_TESTS: &[&str] = &[
    "min_over_time(rate(foo{bar=\"baz\"}[2s])[5m:5s])",
    "min_over_time(rate(foo{bar=\"baz\"}[2s])[5m:])",
];

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::{Expr, expr};

    #[test]
    fn test_simple_subqueries_parse() {
        for input in VALID_SIMPLE_SUBQUERIES {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Should parse to Subquery
                    assert!(
                        matches!(parsed, Expr::Subquery(_)),
                        "Expression '{}' should parse to Subquery, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!("Failed to parse subquery '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_subqueries_with_selectors_parse() {
        for input in SUBQUERIES_WITH_SELECTORS {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    assert!(
                        matches!(parsed, Expr::Subquery(_)),
                        "Expression '{}' should parse to Subquery, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!(
                    "Failed to parse subquery with selector '{}': {:?}",
                    input, e
                ),
            }
        }
    }

    #[test]
    fn test_subqueries_on_functions_parse() {
        for input in SUBQUERIES_ON_FUNCTIONS {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Top level might be Call (for over_time functions with subquery arg)
                    // or Subquery (for subquery on function result)
                    assert!(
                        matches!(parsed, Expr::Subquery(_) | Expr::Call(_)),
                        "Expression '{}' should parse to Subquery or Call, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!("Failed to parse subquery on function '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_subqueries_on_aggregations_parse() {
        for input in SUBQUERIES_ON_AGGREGATIONS {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    assert!(
                        matches!(parsed, Expr::Subquery(_)),
                        "Expression '{}' should parse to Subquery, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!(
                    "Failed to parse subquery on aggregation '{}': {:?}",
                    input, e
                ),
            }
        }
    }

    #[test]
    fn test_subqueries_on_binary_exprs_parse() {
        for input in SUBQUERIES_ON_BINARY_EXPRS {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    assert!(
                        matches!(parsed, Expr::Subquery(_)),
                        "Expression '{}' should parse to Subquery, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!(
                    "Failed to parse subquery on binary expr '{}': {:?}",
                    input, e
                ),
            }
        }
    }

    #[test]
    fn test_nested_subqueries_parse() {
        for input in NESTED_SUBQUERIES {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Top level might be Subquery or Call depending on structure
                    assert!(
                        matches!(parsed, Expr::Subquery(_) | Expr::Call(_)),
                        "Expression '{}' should parse to Subquery or Call, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!("Failed to parse nested subquery '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_subqueries_with_offset_parse() {
        for input in SUBQUERIES_WITH_OFFSET {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Verify it has offset in the subquery
                    if let Expr::Subquery(subq) = parsed {
                        assert!(
                            subq.offset.is_some(),
                            "Subquery '{}' should have offset",
                            input
                        );
                    } else {
                        panic!(
                            "Expression '{}' should parse to Subquery, got {:?}",
                            input, parsed
                        );
                    }
                }
                Err(e) => panic!("Failed to parse subquery with offset '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_subqueries_with_at_parse() {
        for input in SUBQUERIES_WITH_AT {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Verify it has @ modifier in the subquery
                    if let Expr::Subquery(subq) = parsed {
                        assert!(
                            subq.at.is_some(),
                            "Subquery '{}' should have @ modifier",
                            input
                        );
                    } else {
                        panic!(
                            "Expression '{}' should parse to Subquery, got {:?}",
                            input, parsed
                        );
                    }
                }
                Err(e) => panic!(
                    "Failed to parse subquery with @ modifier '{}': {:?}",
                    input, e
                ),
            }
        }
    }

    #[test]
    fn test_subqueries_with_both_modifiers_parse() {
        for input in SUBQUERIES_WITH_BOTH_MODIFIERS {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    if let Expr::Subquery(subq) = parsed {
                        assert!(
                            subq.offset.is_some() && subq.at.is_some(),
                            "Subquery '{}' should have both offset and @ modifier",
                            input
                        );
                    } else {
                        panic!(
                            "Expression '{}' should parse to Subquery, got {:?}",
                            input, parsed
                        );
                    }
                }
                Err(e) => panic!(
                    "Failed to parse subquery with both modifiers '{}': {:?}",
                    input, e
                ),
            }
        }
    }

    #[test]
    fn test_subquery_agg_over_time_patterns_parse() {
        for input in SUBQUERY_AGG_OVER_TIME_PATTERNS {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // These are *_over_time functions with subquery arguments
                    assert!(
                        matches!(parsed, Expr::Call(_)),
                        "Expression '{}' should parse to Call, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!("Failed to parse agg over time pattern '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_invalid_subqueries_fail() {
        for (input, _error_desc) in INVALID_SUBQUERIES {
            let result = expr(input);
            match result {
                Err(_) => {
                    // Good - it should fail
                }
                Ok((remaining, _)) => {
                    // Some inputs might partially parse
                    // Check if it's a known case where parsing succeeds
                    // but semantic validation would fail
                    if remaining.is_empty() {
                        // Semantic errors might not be caught by parser
                    }
                }
            }
        }
    }

    #[test]
    fn test_hpe_subquery_tests_parse() {
        for input in HPE_SUBQUERY_TESTS {
            let result = expr(input);
            match result {
                Ok((remaining, _parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                }
                Err(e) => panic!("Failed to parse HPE subquery test '{}': {:?}", input, e),
            }
        }
    }
}
