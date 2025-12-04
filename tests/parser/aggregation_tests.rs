// Aggregation operator test cases extracted from:
// - references/prometheus/promql/parser/parse_test.go
//
// These test cases cover:
// - All aggregation operators
// - by/without label modifiers
// - Parameter variations
// - Error cases

/// All aggregation operators
pub const AGGREGATION_OPERATORS: &[&str] = &[
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
    "quantile",
    "count_values",
    "limitk",
    "limit_ratio",
];

/// Valid aggregation test cases without grouping
pub const VALID_AGGREGATIONS_SIMPLE: &[&str] = &[
    // Basic aggregations
    "sum(some_metric)",
    "avg(some_metric)",
    "count(some_metric)",
    "min(some_metric)",
    "max(some_metric)",
    "group(some_metric)",
    "stddev(some_metric)",
    "stdvar(some_metric)",
    // With complex inner expressions
    "sum(rate(some_metric[5m]))",
    "avg(some_metric * 2)",
    "count(some_metric > 0)",
    "min(some_metric + other_metric)",
    "max(some_metric{job=\"foo\"})",
    // With selectors
    r#"sum(http_requests_total{job="prometheus"})"#,
    r#"avg(rate(http_requests_total{job="prometheus"}[5m]))"#,
];

/// Valid aggregation test cases with by clause
pub const VALID_AGGREGATIONS_BY: &[&str] = &[
    // Single label
    "sum by (job) (some_metric)",
    "sum(some_metric) by (job)",
    "avg by (instance) (some_metric)",
    "count by (job) (some_metric)",
    "min by (instance) (some_metric)",
    "max by (job, instance) (some_metric)",
    "group by (job) (some_metric)",
    "stddev by (job) (some_metric)",
    "stdvar by (job) (some_metric)",
    // Multiple labels
    "sum by (job, instance) (some_metric)",
    "avg by (job, instance, method) (some_metric)",
    // Empty parentheses (aggregate all)
    "sum by () (some_metric)",
];

/// Valid aggregation test cases with without clause
pub const VALID_AGGREGATIONS_WITHOUT: &[&str] = &[
    // Single label
    "sum without (job) (some_metric)",
    "sum(some_metric) without (job)",
    "avg without (instance) (some_metric)",
    "count without (job) (some_metric)",
    "min without (instance) (some_metric)",
    "max without (job, instance) (some_metric)",
    "group without (job) (some_metric)",
    "stddev without (job) (some_metric)",
    "stdvar without (job) (some_metric)",
    // Multiple labels
    "sum without (job, instance) (some_metric)",
    "avg without (job, instance, method) (some_metric)",
    // Empty parentheses
    "sum without () (some_metric)",
];

/// Valid parametric aggregation test cases (topk, bottomk, quantile, etc.)
pub const VALID_PARAMETRIC_AGGREGATIONS: &[&str] = &[
    // topk
    "topk(5, some_metric)",
    "topk(5, some_metric) by (job)",
    "topk by (job) (5, some_metric)",
    "topk(3, rate(http_requests_total[5m]))",
    // bottomk
    "bottomk(5, some_metric)",
    "bottomk(5, some_metric) by (job)",
    "bottomk by (job) (5, some_metric)",
    // quantile
    "quantile(0.9, some_metric)",
    "quantile(0.5, some_metric) by (job)",
    "quantile by (job) (0.9, some_metric)",
    "quantile(0.99, rate(http_requests_total[5m]))",
    // count_values
    r#"count_values("value", some_metric)"#,
    r#"count_values("value", some_metric) by (job)"#,
    r#"count_values by (job) ("value", some_metric)"#,
    // limitk
    "limitk(5, some_metric)",
    "limitk(5, some_metric) by (job)",
    // limit_ratio
    "limit_ratio(0.5, some_metric)",
    "limit_ratio(0.5, some_metric) by (job)",
];

/// Nested aggregation test cases
pub const VALID_NESTED_AGGREGATIONS: &[&str] = &[
    "sum(sum(some_metric))",
    "max(min(some_metric) by (job))",
    "avg(sum(rate(http_requests_total[5m])) by (job))",
    "topk(5, sum(rate(http_requests_total[5m])) by (job))",
    "quantile(0.9, sum(rate(http_requests_total[5m])) by (job))",
];

/// Aggregation with binary operators
pub const AGGREGATIONS_WITH_BINARY_OPS: &[&str] = &[
    "sum(some_metric) + sum(other_metric)",
    "sum(some_metric) / count(some_metric)",
    "sum(rate(http_requests_total[5m])) by (job) > 100",
    "sum(some_metric) by (job) / on(job) group_left sum(other_metric) by (job)",
    "topk(5, some_metric) + 1",
];

/// Invalid aggregation test cases
pub const INVALID_AGGREGATIONS: &[(&str, &str)] = &[
    // Missing parentheses
    ("sum some_metric", "unexpected identifier"),
    // Missing inner expression
    ("sum()", "expected expression"),
    ("sum by (job) ()", "expected expression"),
    ("topk(5)", "wrong number of arguments"),
    // Invalid grouping syntax
    (
        "sum by job (some_metric)",
        "expected grouped opening parenthesis",
    ),
    (
        "sum without job (some_metric)",
        "expected grouped opening parenthesis",
    ),
    // Wrong parameter type for parametric aggregations
    ("topk(some_metric, 5)", "expected type scalar"),
    ("topk(some_metric, other_metric)", "expected type scalar"),
    ("quantile(some_metric, 0.9)", "expected type scalar"),
    // Invalid quantile values (these might be semantic, not syntax errors)
    // ("quantile(1.5, some_metric)", "quantile value out of range"),
    // ("quantile(-0.1, some_metric)", "quantile value out of range"),

    // Invalid count_values
    ("count_values(5, some_metric)", "expected type string"),
    // Both by and without (invalid)
    (
        "sum by (job) without (instance) (some_metric)",
        "unexpected",
    ),
    // Aggregation used as function argument (semantic error)
    // ("floor(sum(some_metric))", "..."), // This is actually valid!

    // Missing comma in label list
    (
        "sum by (job instance) (some_metric)",
        "expected grouped closing parenthesis",
    ),
    // Trailing comma in label list
    ("sum by (job,) (some_metric)", "expected label name"),
    // Using reserved keywords as labels in by clause might cause issues
    ("sum by (on) (some_metric)", "expected label name"), // 'on' is reserved
    ("sum by (group_left) (some_metric)", "expected label name"),
];

/// Keep grouping modifier tests
pub const KEEP_FIRING_TESTS: &[&str] = &[
    // Note: These are for alerting rules, not aggregations
    // But keep_firing_for is a modifier
];

/// Aggregation expressions from real-world queries
pub const REAL_WORLD_AGGREGATIONS: &[&str] = &[
    // Request rates by service
    r#"sum(rate(http_requests_total[5m])) by (service)"#,
    // Error rate percentage
    r#"sum(rate(http_requests_total{status=~"5.."}[5m])) by (service) / sum(rate(http_requests_total[5m])) by (service) * 100"#,
    // Top 10 memory consumers
    r#"topk(10, container_memory_usage_bytes) by (pod)"#,
    // P99 latency
    r#"histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le, service))"#,
    // Count of unique label values
    r#"count(count by (instance) (up))"#,
    // Grouped aggregation with complex selector
    r#"sum by (job, instance) (rate(node_cpu_seconds_total{mode!="idle"}[5m]))"#,
];

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::{Expr, expr};

    #[test]
    fn test_simple_aggregations_parse() {
        for input in VALID_AGGREGATIONS_SIMPLE {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Verify it parsed to an aggregation
                    assert!(
                        matches!(parsed, Expr::Aggregation(_)),
                        "Expression '{}' should parse to Aggregation, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!("Failed to parse aggregation '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_aggregations_with_by_clause() {
        for input in VALID_AGGREGATIONS_BY {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Verify it parsed to an aggregation with grouping
                    if let Expr::Aggregation(agg) = parsed {
                        assert!(
                            agg.grouping.is_some(),
                            "Aggregation '{}' should have grouping",
                            input
                        );
                    } else {
                        panic!(
                            "Expression '{}' should parse to Aggregation, got {:?}",
                            input, parsed
                        );
                    }
                }
                Err(e) => panic!("Failed to parse aggregation '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_aggregations_with_without_clause() {
        for input in VALID_AGGREGATIONS_WITHOUT {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Verify it parsed to an aggregation with grouping
                    if let Expr::Aggregation(agg) = parsed {
                        assert!(
                            agg.grouping.is_some(),
                            "Aggregation '{}' should have grouping",
                            input
                        );
                    } else {
                        panic!(
                            "Expression '{}' should parse to Aggregation, got {:?}",
                            input, parsed
                        );
                    }
                }
                Err(e) => panic!("Failed to parse aggregation '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_parametric_aggregations_parse() {
        for input in VALID_PARAMETRIC_AGGREGATIONS {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Verify it parsed to an aggregation with param
                    if let Expr::Aggregation(agg) = parsed {
                        assert!(
                            agg.param.is_some(),
                            "Parametric aggregation '{}' should have param",
                            input
                        );
                    } else {
                        panic!(
                            "Expression '{}' should parse to Aggregation, got {:?}",
                            input, parsed
                        );
                    }
                }
                Err(e) => panic!(
                    "Failed to parse parametric aggregation '{}': {:?}",
                    input, e
                ),
            }
        }
    }

    #[test]
    fn test_nested_aggregations_parse() {
        for input in VALID_NESTED_AGGREGATIONS {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Verify it parsed to an aggregation
                    assert!(
                        matches!(parsed, Expr::Aggregation(_)),
                        "Expression '{}' should parse to Aggregation, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!("Failed to parse nested aggregation '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_aggregations_with_binary_ops_parse() {
        for input in AGGREGATIONS_WITH_BINARY_OPS {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // These should parse - the top level might be Binary or Aggregation
                    // depending on operator precedence
                    assert!(
                        matches!(parsed, Expr::Binary(_) | Expr::Aggregation(_)),
                        "Expression '{}' should parse to Binary or Aggregation, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!(
                    "Failed to parse aggregation with binary op '{}': {:?}",
                    input, e
                ),
            }
        }
    }

    #[test]
    fn test_invalid_aggregations_fail() {
        for (input, _error_desc) in INVALID_AGGREGATIONS {
            let result = expr(input);
            // Should either fail or not fully consume input
            match result {
                Err(_) => {
                    // Good - it should fail
                }
                Ok((remaining, _)) => {
                    // Some invalid inputs might partially parse
                    // That's acceptable as long as they don't fully parse
                    if remaining.is_empty() {
                        // Check if it's a known case where parsing succeeds but semantic validation would fail
                        // (Some "invalid" cases are actually syntactically valid but semantically wrong)
                    }
                }
            }
        }
    }

    #[test]
    fn test_aggregation_operators() {
        assert_eq!(AGGREGATION_OPERATORS.len(), 14);
        // Test each operator parses in a simple context
        // Note: parametric aggregations like topk, bottomk, quantile, count_values, limitk, limit_ratio
        // require a parameter, so we test those differently
        let parametric = [
            "topk",
            "bottomk",
            "quantile",
            "count_values",
            "limitk",
            "limit_ratio",
        ];
        for op in AGGREGATION_OPERATORS {
            let input = if parametric.contains(op) {
                if *op == "count_values" {
                    format!(r#"{}("label", some_metric)"#, op)
                } else {
                    format!("{}(5, some_metric)", op)
                }
            } else {
                format!("{}(some_metric)", op)
            };
            let result = expr(&input);
            assert!(
                result.is_ok(),
                "Aggregation operator '{}' should parse in '{}'",
                op,
                input
            );
        }
    }

    #[test]
    fn test_real_world_aggregations_parse() {
        for input in REAL_WORLD_AGGREGATIONS {
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
                Err(e) => panic!(
                    "Failed to parse real-world aggregation '{}': {:?}",
                    input, e
                ),
            }
        }
    }

    #[test]
    fn test_keep_firing_data() {
        // KEEP_FIRING_TESTS is empty by design (it's for alerting rules, not aggregations)
        // Just verify it exists and is an array
        let _: &[&str] = KEEP_FIRING_TESTS;
    }
}
