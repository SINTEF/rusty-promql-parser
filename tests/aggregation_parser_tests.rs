// Integration tests for aggregation expression parsing
//
// These tests verify that the expression parser correctly handles aggregation
// expressions from the Go Prometheus parser test cases.

mod parser {
    pub mod aggregation_tests;
}

use parser::aggregation_tests::{
    AGGREGATION_OPERATORS, VALID_AGGREGATIONS_BY, VALID_AGGREGATIONS_SIMPLE,
    VALID_AGGREGATIONS_WITHOUT, VALID_PARAMETRIC_AGGREGATIONS,
};
use rusty_promql_parser::parser::aggregation::GroupingAction;
use rusty_promql_parser::{Expr, expr};

/// Test that simple aggregation expressions parse successfully
#[test]
fn test_simple_aggregations_from_test_data() {
    for input in VALID_AGGREGATIONS_SIMPLE {
        let result = expr(input);
        assert!(
            result.is_ok(),
            "Failed to parse simple aggregation: {:?}\nError: {:?}",
            input,
            result.err()
        );
        let (_, e) = result.unwrap();
        assert!(
            matches!(e, Expr::Aggregation(_)),
            "Expected Aggregation for '{}'",
            input
        );
    }
}

/// Test that by-clause aggregations parse successfully
#[test]
fn test_by_aggregations_from_test_data() {
    for input in VALID_AGGREGATIONS_BY {
        let result = expr(input);
        assert!(
            result.is_ok(),
            "Failed to parse by-clause aggregation: {:?}\nError: {:?}",
            input,
            result.err()
        );

        let (_, e) = result.unwrap();
        if let Expr::Aggregation(agg) = e {
            // Should have grouping with By action
            if let Some(ref g) = agg.grouping {
                assert_eq!(
                    g.action,
                    GroupingAction::By,
                    "Expected By grouping for: {:?}",
                    input
                );
            }
        }
    }
}

/// Test that without-clause aggregations parse successfully
#[test]
fn test_without_aggregations_from_test_data() {
    for input in VALID_AGGREGATIONS_WITHOUT {
        let result = expr(input);
        assert!(
            result.is_ok(),
            "Failed to parse without-clause aggregation: {:?}\nError: {:?}",
            input,
            result.err()
        );

        let (_, e) = result.unwrap();
        if let Expr::Aggregation(agg) = e {
            // Should have grouping with Without action
            if let Some(ref g) = agg.grouping {
                assert_eq!(
                    g.action,
                    GroupingAction::Without,
                    "Expected Without grouping for: {:?}",
                    input
                );
            }
        }
    }
}

/// Test that parametric aggregations parse successfully
#[test]
fn test_parametric_aggregations_from_test_data() {
    for input in VALID_PARAMETRIC_AGGREGATIONS {
        let result = expr(input);
        assert!(
            result.is_ok(),
            "Failed to parse parametric aggregation: {:?}\nError: {:?}",
            input,
            result.err()
        );

        let (_, e) = result.unwrap();
        if let Expr::Aggregation(agg) = e {
            // Parametric aggregations should have a param
            assert!(
                agg.param.is_some(),
                "Expected param for parametric aggregation: {:?}",
                input
            );
        }
    }
}

/// Test that all aggregation operators are recognized
#[test]
fn test_all_aggregation_operators() {
    // Parametric aggregations require a first parameter
    let parametric_ops = [
        "topk",
        "bottomk",
        "quantile",
        "count_values",
        "limitk",
        "limit_ratio",
    ];

    for op in AGGREGATION_OPERATORS {
        let input = if parametric_ops.contains(op) {
            // Parametric: use appropriate first arg
            if *op == "count_values" {
                format!(r#"{}("label", metric)"#, op)
            } else {
                format!("{}(5, metric)", op)
            }
        } else {
            format!("{}(metric)", op)
        };

        let result = expr(&input);
        assert!(
            result.is_ok(),
            "Failed to parse aggregation operator: {:?}\nInput: {:?}\nError: {:?}",
            op,
            input,
            result.err()
        );

        let (_, e) = result.unwrap();
        if let Expr::Aggregation(agg) = e {
            assert_eq!(
                agg.op.to_lowercase(),
                op.to_lowercase(),
                "Operator name mismatch"
            );
        }
    }
}

/// Test grouping modifier positions
#[test]
fn test_grouping_before_and_after() {
    // Grouping before expression
    let (_, e) = expr("sum by (job) (metric)").unwrap();
    if let Expr::Aggregation(agg) = e {
        assert!(agg.grouping.is_some());
        assert_eq!(agg.grouping.unwrap().action, GroupingAction::By);
    }

    // Grouping after expression
    let (_, e) = expr("sum(metric) by (job)").unwrap();
    if let Expr::Aggregation(agg) = e {
        assert!(agg.grouping.is_some());
        assert_eq!(agg.grouping.unwrap().action, GroupingAction::By);
    }
}

/// Test Display implementation
#[test]
fn test_aggregation_display() {
    let test_cases = &["sum(metric)", "avg(metric)", "topk(5, metric)"];

    for input in test_cases {
        let (_, e) = expr(input).unwrap();
        let display = format!("{}", e);
        // Check it contains the operator
        assert!(
            display.to_lowercase().contains("sum")
                || display.to_lowercase().contains("avg")
                || display.to_lowercase().contains("topk"),
            "Display should contain operator for '{}': got '{}'",
            input,
            display
        );
    }
}

/// Test multiple labels in grouping
#[test]
fn test_multiple_labels_in_grouping() {
    let (_, e) = expr("sum by (job, instance, method) (metric)").unwrap();
    if let Expr::Aggregation(agg) = e {
        let grouping = agg.grouping.unwrap();
        assert_eq!(grouping.labels, vec!["job", "instance", "method"]);
    }
}

/// Test empty label list in grouping
#[test]
fn test_empty_labels_in_grouping() {
    let (_, e) = expr("sum by () (metric)").unwrap();
    if let Expr::Aggregation(agg) = e {
        let grouping = agg.grouping.unwrap();
        assert!(grouping.labels.is_empty());
    }
}
