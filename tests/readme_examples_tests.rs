//! Tests for README.md examples
//!
//! These tests document the expected AST output for the examples in README.md.
//! They serve as both documentation and regression tests.

use rusty_promql_parser::parser::expr::expr;

/// A metric with label filtering
#[test]
fn test_metric_with_labels() {
    let input = r#"go_gc_duration_seconds{instance="localhost:9090", job="alertmanager"}"#;

    let (rest, ast) = expr(input).expect("failed to parse");
    assert!(rest.is_empty());

    let expected = r#"VectorSelector(VectorSelector { name: Some("go_gc_duration_seconds"), matchers: [LabelMatcher { name: "instance", op: Equal, value: "localhost:9090" }, LabelMatcher { name: "job", op: Equal, value: "alertmanager" }], offset: None, at: None })"#;
    assert_eq!(format!("{:?}", ast), expected);
}

/// Aggregation operators
#[test]
fn test_aggregation_operators() {
    let input = r#"sum by (app, proc) (
  instance_memory_limit_bytes - instance_memory_usage_bytes
) / 1024 / 1024"#;

    let (rest, ast) = expr(input).expect("failed to parse");
    assert!(rest.is_empty());

    let expected = r#"Binary(BinaryExpr { op: Div, lhs: Binary(BinaryExpr { op: Div, lhs: Aggregation(Aggregation { op: "sum", expr: Binary(BinaryExpr { op: Sub, lhs: VectorSelector(VectorSelector { name: Some("instance_memory_limit_bytes"), matchers: [], offset: None, at: None }), rhs: VectorSelector(VectorSelector { name: Some("instance_memory_usage_bytes"), matchers: [], offset: None, at: None }), modifier: None }), param: None, grouping: Some(Grouping { action: By, labels: ["app", "proc"] }) }), rhs: Number(1024.0), modifier: None }), rhs: Number(1024.0), modifier: None })"#;
    assert_eq!(format!("{:?}", ast), expected);
}
