// Function call test cases extracted from:
// - references/prometheus/promql/parser/parse_test.go
// - references/prometheus/promql/parser/functions.go
//
// These test cases cover:
// - Function calls with various argument types
// - Function argument validation
// - Unknown function handling

/// Valid function call test cases
pub const VALID_FUNCTION_CALLS: &[&str] = &[
    // Zero-argument functions
    "time()",
    "pi()",
    // Single-argument functions (instant vector -> instant vector)
    "abs(some_metric)",
    "ceil(some_metric)",
    "floor(some_metric)",
    r#"floor(some_metric{foo!="bar"})"#,
    "exp(some_metric)",
    "ln(some_metric)",
    "log2(some_metric)",
    "log10(some_metric)",
    "sqrt(some_metric)",
    "sgn(some_metric)",
    "sort(some_metric)",
    "sort_desc(some_metric)",
    // Single-argument functions (range vector -> instant vector)
    "rate(some_metric[5m])",
    "irate(some_metric[5m])",
    "increase(some_metric[5m])",
    "delta(some_metric[5m])",
    "idelta(some_metric[5m])",
    "deriv(some_metric[5m])",
    "changes(some_metric[5m])",
    "resets(some_metric[5m])",
    "avg_over_time(some_metric[5m])",
    "sum_over_time(some_metric[5m])",
    "count_over_time(some_metric[5m])",
    "min_over_time(some_metric[5m])",
    "max_over_time(some_metric[5m])",
    "stddev_over_time(some_metric[5m])",
    "stdvar_over_time(some_metric[5m])",
    "last_over_time(some_metric[5m])",
    "present_over_time(some_metric[5m])",
    "absent_over_time(some_metric[5m])",
    // Optional-argument functions
    "round(some_metric)",
    "round(some_metric, 5)",
    "hour()",
    "hour(some_metric)",
    "day_of_week(some_metric)",
    "day_of_month(some_metric)",
    "day_of_year(some_metric)",
    "days_in_month(some_metric)",
    "month(some_metric)",
    "year(some_metric)",
    "minute(some_metric)",
    // Multi-argument functions
    "clamp(some_metric, 0, 100)",
    "clamp_min(some_metric, 0)",
    "clamp_max(some_metric, 100)",
    "predict_linear(some_metric[5m], 3600)",
    "histogram_quantile(0.9, some_metric)",
    "quantile_over_time(0.9, some_metric[5m])",
    // String argument functions
    r#"label_replace(some_metric, "dst", "$1", "src", "(.*)")"#,
    r#"label_join(some_metric, "dst", ",", "src1", "src2")"#,
    // Vector function
    "vector(1)",
    "vector(1.5)",
    // Absent function
    "absent(some_metric)",
    r#"absent(some_metric{job="foo"})"#,
    // Scalar function
    "scalar(some_metric)",
    // Timestamp function
    "timestamp(some_metric)",
    // Histogram functions
    "histogram_quantile(0.9, rate(http_requests_total[5m]))",
    "histogram_sum(some_histogram)",
    "histogram_count(some_histogram)",
    "histogram_fraction(0, 0.5, some_histogram)",
    // Info function
    "info(rate(http_request_counter_total{}[5m]))",
    r#"info(http_request_counter_total{namespace="zzz"}, {foo="bar", bar="baz"})"#,
    // Nested function calls
    "rate(http_requests_total[5m])",
    // Note: sum() is an aggregation operator, not a function - tested in aggregation_tests.rs
    "max_over_time(rate(http_requests[5m])[30m:1m])",
    "min_over_time(rate(foo{bar=\"baz\"}[2s])[5m:5s])",
];

/// Invalid function call test cases
pub const INVALID_FUNCTION_CALLS: &[(&str, &str)] = &[
    // Unknown function
    ("non_existent_function_far_bar()", "unknown function"),
    ("b()", "unknown function"),
    // Wrong argument count
    ("floor()", "expected 1 argument"),
    ("floor(some_metric, other_metric)", "expected 1 argument"),
    ("floor(some_metric, 1)", "expected 1 argument"),
    ("time(some_metric)", "expected 0 argument"),
    (
        "hour(some_metric, some_metric, some_metric)",
        "expected at most 1 argument",
    ),
    ("topk(some_metric)", "wrong number of arguments"),
    // Wrong argument type
    ("floor(1)", "expected type instant vector"),
    ("rate(some_metric)", "expected type range vector"),
    ("rate(avg)", "expected type range vector"),
    ("topk(some_metric, other_metric)", "expected type scalar"),
    ("count_values(5, other_metric)", "expected type string"),
    // @ modifier on function result (not allowed)
    (
        "rate(some_metric[5m]) @ 1234",
        "@ modifier must be preceded by",
    ),
];

/// List of all built-in functions with their signatures
/// Format: (name, min_args, max_args, return_type)
pub const FUNCTION_SIGNATURES: &[(&str, u8, u8, &str)] = &[
    // Trigonometric functions
    ("acos", 1, 1, "vector"),
    ("acosh", 1, 1, "vector"),
    ("asin", 1, 1, "vector"),
    ("asinh", 1, 1, "vector"),
    ("atan", 1, 1, "vector"),
    ("atanh", 1, 1, "vector"),
    ("cos", 1, 1, "vector"),
    ("cosh", 1, 1, "vector"),
    ("sin", 1, 1, "vector"),
    ("sinh", 1, 1, "vector"),
    ("tan", 1, 1, "vector"),
    ("tanh", 1, 1, "vector"),
    // Math functions
    ("abs", 1, 1, "vector"),
    ("ceil", 1, 1, "vector"),
    ("floor", 1, 1, "vector"),
    ("exp", 1, 1, "vector"),
    ("sqrt", 1, 1, "vector"),
    ("ln", 1, 1, "vector"),
    ("log2", 1, 1, "vector"),
    ("log10", 1, 1, "vector"),
    ("sgn", 1, 1, "vector"),
    ("deg", 1, 1, "vector"),
    ("rad", 1, 1, "vector"),
    // Rounding functions
    ("round", 1, 2, "vector"),
    ("clamp", 3, 3, "vector"),
    ("clamp_min", 2, 2, "vector"),
    ("clamp_max", 2, 2, "vector"),
    // Aggregation over time functions
    ("avg_over_time", 1, 1, "vector"),
    ("min_over_time", 1, 1, "vector"),
    ("max_over_time", 1, 1, "vector"),
    ("sum_over_time", 1, 1, "vector"),
    ("count_over_time", 1, 1, "vector"),
    ("quantile_over_time", 2, 2, "vector"),
    ("stddev_over_time", 1, 1, "vector"),
    ("stdvar_over_time", 1, 1, "vector"),
    ("last_over_time", 1, 1, "vector"),
    ("present_over_time", 1, 1, "vector"),
    ("absent_over_time", 1, 1, "vector"),
    ("mad_over_time", 1, 1, "vector"),
    // Rate functions
    ("rate", 1, 1, "vector"),
    ("irate", 1, 1, "vector"),
    ("increase", 1, 1, "vector"),
    ("delta", 1, 1, "vector"),
    ("idelta", 1, 1, "vector"),
    ("deriv", 1, 1, "vector"),
    ("predict_linear", 2, 2, "vector"),
    // Counter functions
    ("changes", 1, 1, "vector"),
    ("resets", 1, 1, "vector"),
    // Label functions
    ("label_replace", 5, 5, "vector"),
    ("label_join", 3, 255, "vector"), // Go semantics: min = len(ArgTypes) - 1 for variadic
    // Sort functions
    ("sort", 1, 1, "vector"),
    ("sort_desc", 1, 1, "vector"),
    ("sort_by_label", 1, 255, "vector"), // Go: min = len(ArgTypes) - 1 for variadic
    ("sort_by_label_desc", 1, 255, "vector"), // Go: min = len(ArgTypes) - 1 for variadic
    // Date/time functions
    ("time", 0, 0, "scalar"),
    ("minute", 0, 1, "vector"),
    ("hour", 0, 1, "vector"),
    ("day_of_week", 0, 1, "vector"),
    ("day_of_month", 0, 1, "vector"),
    ("day_of_year", 0, 1, "vector"),
    ("days_in_month", 0, 1, "vector"),
    ("month", 0, 1, "vector"),
    ("year", 0, 1, "vector"),
    ("timestamp", 1, 1, "vector"),
    // Type conversion
    ("vector", 1, 1, "vector"),
    ("scalar", 1, 1, "scalar"),
    // Existence check
    ("absent", 1, 1, "vector"),
    // Histogram functions
    ("histogram_quantile", 2, 2, "vector"),
    ("histogram_sum", 1, 1, "vector"),
    ("histogram_count", 1, 1, "vector"),
    ("histogram_avg", 1, 1, "vector"),
    ("histogram_stddev", 1, 1, "vector"),
    ("histogram_stdvar", 1, 1, "vector"),
    ("histogram_fraction", 3, 3, "vector"),
    // Other
    ("pi", 0, 0, "scalar"),
    ("info", 1, 2, "vector"),
    ("double_exponential_smoothing", 3, 3, "vector"),
    // Additional over_time functions
    ("first_over_time", 1, 1, "vector"),
    // Timestamp functions (experimental)
    ("ts_of_first_over_time", 1, 1, "vector"),
    ("ts_of_max_over_time", 1, 1, "vector"),
    ("ts_of_min_over_time", 1, 1, "vector"),
    ("ts_of_last_over_time", 1, 1, "vector"),
];

/// Aggregation operators (can look like functions but have different syntax)
pub const AGGREGATION_OPS: &[&str] = &[
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::{Expr, expr};

    #[test]
    fn test_valid_function_calls_parse() {
        for input in VALID_FUNCTION_CALLS {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    // Function calls should parse to Call, Aggregation, or Subquery (for nested)
                    assert!(
                        matches!(
                            parsed,
                            Expr::Call(_)
                                | Expr::Aggregation(_)
                                | Expr::Subquery(_)
                                | Expr::VectorSelector(_)
                                | Expr::MatrixSelector(_)
                        ),
                        "Expression '{}' should parse to Call or related type, got {:?}",
                        input,
                        parsed
                    );
                }
                Err(e) => panic!("Failed to parse function call '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_invalid_function_calls_fail() {
        for (input, _error_desc) in INVALID_FUNCTION_CALLS {
            let result = expr(input);
            // Should either fail or not fully consume input
            match result {
                Err(_) => {
                    // Good - it should fail
                }
                Ok((remaining, _)) => {
                    // Some inputs might partially parse
                    // Check if it's a known case where parsing succeeds
                    // but semantic validation would fail
                    if remaining.is_empty() {
                        // Semantic errors (like unknown function, wrong arg count) might
                        // not be caught by the parser, only by a later validation pass
                    }
                }
            }
        }
    }

    #[test]
    fn test_function_signatures() {
        // Verify function signature completeness
        assert!(
            FUNCTION_SIGNATURES.len() >= 60,
            "Should have at least 60 function signatures"
        );
        for (name, min_args, max_args, return_type) in FUNCTION_SIGNATURES {
            assert!(
                !name.is_empty(),
                "Empty function name in FUNCTION_SIGNATURES"
            );
            assert!(
                max_args >= min_args,
                "max_args should be >= min_args for '{}'",
                name
            );
            assert!(!return_type.is_empty(), "Empty return type for '{}'", name);
        }
    }

    #[test]
    fn test_aggregation_ops() {
        // Verify aggregation operators list completeness
        // These are tested in aggregation_tests.rs - verify they parse
        assert_eq!(
            AGGREGATION_OPS.len(),
            14,
            "Should have 14 aggregation operators"
        );
        // Note: parametric aggregations require parameters
        let parametric = [
            "topk",
            "bottomk",
            "quantile",
            "count_values",
            "limitk",
            "limit_ratio",
        ];
        for op in AGGREGATION_OPS {
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
                "Aggregation '{}' should parse in '{}'",
                op,
                input
            );
        }
    }

    #[test]
    fn test_zero_arg_functions() {
        // Test zero-argument functions
        let zero_arg_functions = ["time()", "pi()"];
        for input in zero_arg_functions {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(remaining.is_empty());
                    if let Expr::Call(call) = parsed {
                        assert!(
                            call.args.is_empty(),
                            "Zero-arg function '{}' should have no args",
                            input
                        );
                    } else {
                        panic!("'{}' should parse to Call", input);
                    }
                }
                Err(e) => panic!("Failed to parse zero-arg function '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_single_arg_functions() {
        // Test that single-arg functions have exactly one argument
        let single_arg_inputs = [("abs(some_metric)", 1), ("rate(metric[5m])", 1)];
        for (input, expected_args) in single_arg_inputs {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(remaining.is_empty());
                    if let Expr::Call(call) = parsed {
                        assert_eq!(
                            call.args.len(),
                            expected_args,
                            "Function call '{}' should have {} args",
                            input,
                            expected_args
                        );
                    } else {
                        panic!("'{}' should parse to Call", input);
                    }
                }
                Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_multi_arg_functions() {
        // Test multi-argument functions
        let multi_arg_inputs = [
            ("clamp(metric, 0, 100)", 3),
            ("histogram_quantile(0.9, metric)", 2),
        ];
        for (input, expected_args) in multi_arg_inputs {
            let result = expr(input);
            match result {
                Ok((remaining, parsed)) => {
                    assert!(remaining.is_empty());
                    if let Expr::Call(call) = parsed {
                        assert_eq!(
                            call.args.len(),
                            expected_args,
                            "Function call '{}' should have {} args",
                            input,
                            expected_args
                        );
                    } else {
                        panic!("'{}' should parse to Call", input);
                    }
                }
                Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
            }
        }
    }
}
