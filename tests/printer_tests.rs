//! Printer / Display tests for PromQL expressions
//!
//! These tests verify that parsed expressions produce canonical Display output.
//! The pattern is: parse(input) -> format!("{}", expr) -> assert expected output.
//!
//! Based on test cases from:
//! - references/prometheus/promql/parser/printer_test.go

use rusty_promql_parser::parser::expr;

/// Helper to test expression display canonicalization
///
/// Parses the input, formats it with Display, and compares to expected output.
/// If expected is None, the output should match the input.
fn assert_expr_string(input: &str, expected: Option<&str>) {
    let result = expr(input);
    assert!(
        result.is_ok(),
        "Failed to parse '{}': {:?}",
        input,
        result.err()
    );
    let (rest, e) = result.unwrap();
    assert!(
        rest.trim().is_empty(),
        "Remaining input after parsing '{}': '{}'",
        input,
        rest
    );

    let output = format!("{}", e);
    let expected_str = expected.unwrap_or(input);
    assert_eq!(
        output, expected_str,
        "Display mismatch for input '{}'\n  got:      '{}'\n  expected: '{}'",
        input, output, expected_str
    );
}

// =============================================================================
// Aggregation Display Tests
// =============================================================================

#[test]
fn test_aggregation_simple() {
    assert_expr_string("sum(metric)", None);
    assert_expr_string("avg(metric)", None);
    assert_expr_string("count(metric)", None);
    assert_expr_string("min(metric)", None);
    assert_expr_string("max(metric)", None);
}

#[test]
fn test_aggregation_by_empty() {
    // Our implementation keeps empty by() - unlike Go which removes it
    assert_expr_string(
        r#"sum by() (task:errors:rate10s{job="s"})"#,
        Some(r#"sum by () (task:errors:rate10s{job="s"})"#),
    );
}

#[test]
fn test_aggregation_by_with_labels() {
    assert_expr_string(
        r#"sum by(code) (task:errors:rate10s{job="s"})"#,
        Some(r#"sum by (code) (task:errors:rate10s{job="s"})"#),
    );
}

#[test]
fn test_aggregation_without_empty() {
    // Empty without() should be preserved per Go reference
    assert_expr_string(
        r#"sum without() (task:errors:rate10s{job="s"})"#,
        Some(r#"sum without () (task:errors:rate10s{job="s"})"#),
    );
}

#[test]
fn test_aggregation_without_with_labels() {
    assert_expr_string(
        r#"sum without(instance) (task:errors:rate10s{job="s"})"#,
        Some(r#"sum without (instance) (task:errors:rate10s{job="s"})"#),
    );
}

#[test]
fn test_aggregation_topk() {
    assert_expr_string(r#"topk(5, task:errors:rate10s{job="s"})"#, None);
}

#[test]
fn test_aggregation_count_values() {
    assert_expr_string(
        r#"count_values("value", task:errors:rate10s{job="s"})"#,
        None,
    );
}

#[test]
fn test_aggregation_quantile() {
    assert_expr_string(r#"quantile(0.9, metric)"#, None);
}

// =============================================================================
// Binary Expression Display Tests
// =============================================================================

#[test]
fn test_binary_simple() {
    assert_expr_string("1 + 2", None);
    assert_expr_string("foo + bar", None);
    assert_expr_string("foo - bar", None);
    assert_expr_string("foo * bar", None);
    assert_expr_string("foo / bar", None);
}

#[test]
fn test_binary_comparison_bool() {
    assert_expr_string("up > bool 0", None);
    assert_expr_string("foo == bool bar", None);
}

#[test]
fn test_binary_on_empty() {
    // Empty on() should include space around parens
    assert_expr_string("a - on() c", Some("a - on () c"));
}

#[test]
fn test_binary_on_with_labels() {
    assert_expr_string("a - on(b) c", Some("a - on (b) c"));
}

#[test]
fn test_binary_on_group_left() {
    assert_expr_string(
        "a - on(b) group_left(x) c",
        Some("a - on (b) group_left (x) c"),
    );
}

#[test]
fn test_binary_on_group_left_multiple() {
    assert_expr_string(
        "a - on(b) group_left(x, y) c",
        Some("a - on (b) group_left (x, y) c"),
    );
}

#[test]
fn test_binary_on_group_left_empty() {
    assert_expr_string("a - on(b) group_left c", Some("a - on (b) group_left c"));
}

#[test]
fn test_binary_on_group_left_empty_parens() {
    assert_expr_string(
        "a - on(b) group_left() (c)",
        Some("a - on (b) group_left (c)"),
    );
}

#[test]
fn test_binary_ignoring() {
    assert_expr_string("a - ignoring(b) c", Some("a - ignoring (b) c"));
}

#[test]
fn test_binary_ignoring_empty() {
    // Our implementation keeps empty ignoring() - unlike Go which removes it
    assert_expr_string("a - ignoring() c", Some("a - ignoring () c"));
}

#[test]
fn test_binary_set_operators() {
    assert_expr_string("foo and bar", None);
    assert_expr_string("foo or bar", None);
    assert_expr_string("foo unless bar", None);
}

// =============================================================================
// Offset Modifier Display Tests
// =============================================================================

#[test]
fn test_offset_vector_selector() {
    assert_expr_string("a offset 1m", None);
}

#[test]
fn test_offset_negative() {
    // Note: Display may normalize negative offset representation
    assert_expr_string("a offset -7m", None);
}

#[test]
fn test_offset_matrix_selector() {
    assert_expr_string(r#"a{c="d"}[5m] offset 1m"#, None);
    assert_expr_string("a[5m] offset 1m", None);
    assert_expr_string("a[12m] offset -3m", None);
}

#[test]
fn test_offset_subquery() {
    assert_expr_string("a[1h:5m] offset 1m", None);
}

// =============================================================================
// @ Modifier Display Tests
// =============================================================================

#[test]
fn test_at_timestamp() {
    // Timestamps should be formatted with 3 decimal places
    assert_expr_string("a @ 10", Some("a @ 10.000"));
    assert_expr_string("a[1m] @ 10", Some("a[1m] @ 10.000"));
}

#[test]
fn test_at_start_end() {
    assert_expr_string("a @ start()", None);
    assert_expr_string("a @ end()", None);
    assert_expr_string("a[1m] @ start()", None);
    assert_expr_string("a[1m] @ end()", None);
}

// =============================================================================
// Vector Selector Display Tests
// =============================================================================

#[test]
fn test_vector_selector_simple() {
    assert_expr_string("foo", None);
    assert_expr_string("http_requests_total", None);
}

#[test]
fn test_vector_selector_with_name_label() {
    // Selector with __name__ label should display as metric name
    assert_expr_string(r#"{__name__="a"}"#, Some("a"));
}

#[test]
fn test_vector_selector_label_matchers() {
    assert_expr_string(r#"a{b!="c"}[1m]"#, None);
    assert_expr_string(r#"a{b=~"c"}[1m]"#, None);
    assert_expr_string(r#"a{b!~"c"}[1m]"#, None);
}

// =============================================================================
// Subquery Display Tests
// =============================================================================

#[test]
fn test_subquery_with_step() {
    assert_expr_string("metric[5m:1m]", None);
    assert_expr_string("metric[1h:5m]", None);
}

#[test]
fn test_subquery_without_step() {
    assert_expr_string("metric[5m:]", None);
    assert_expr_string("metric[1h:]", None);
}

#[test]
fn test_subquery_with_offset() {
    assert_expr_string("metric[5m:1m] offset 10m", None);
}

#[test]
fn test_subquery_with_at() {
    assert_expr_string("metric[5m:1m] @ start()", None);
    assert_expr_string("metric[5m:1m] @ end()", None);
}

#[test]
fn test_subquery_nested() {
    // Subquery on a rate result
    assert_expr_string("rate(metric[5m])[30m:1m]", None);
}

// =============================================================================
// Function Call Display Tests
// =============================================================================

#[test]
fn test_function_call_single_arg() {
    assert_expr_string("abs(metric)", None);
    assert_expr_string("ceil(metric)", None);
    assert_expr_string("floor(metric)", None);
}

#[test]
fn test_function_call_multiple_args() {
    assert_expr_string("clamp(metric, 0, 100)", None);
    assert_expr_string("clamp_min(metric, 0)", None);
    assert_expr_string("clamp_max(metric, 100)", None);
}

#[test]
fn test_function_call_range_vector() {
    assert_expr_string("rate(http_requests[5m])", None);
    assert_expr_string("irate(http_requests[5m])", None);
    assert_expr_string("increase(http_requests[5m])", None);
}

#[test]
fn test_function_call_predict_linear() {
    assert_expr_string("predict_linear(foo[1h], 3000)", None);
}

// =============================================================================
// Complex Expression Display Tests
// =============================================================================

#[test]
fn test_complex_aggregation_with_function() {
    assert_expr_string("sum(rate(http_requests[5m]))", None);
    assert_expr_string(
        "avg(rate(http_requests[5m])) by (job)",
        Some("avg by (job) (rate(http_requests[5m]))"),
    );
}

#[test]
fn test_complex_nested_binary() {
    assert_expr_string("1 + 2 * 3", None);
    assert_expr_string("(1 + 2) * 3", None);
}

#[test]
fn test_complex_unary() {
    assert_expr_string("-metric", None);
    assert_expr_string("+metric", None);
    assert_expr_string("-1", None);
}

// =============================================================================
// Number Display Tests
// =============================================================================

#[test]
fn test_number_integer() {
    assert_expr_string("42", None);
    assert_expr_string("1048576", None);
}

#[test]
fn test_number_float() {
    assert_expr_string("3.14", None);
}

#[test]
fn test_number_special() {
    assert_expr_string("Inf", None);
    assert_expr_string("-Inf", Some("-Inf"));
    assert_expr_string("NaN", None);
}

// =============================================================================
// String Display Tests
// =============================================================================

#[test]
fn test_string_literal() {
    assert_expr_string(r#""hello""#, None);
    assert_expr_string(r#""hello world""#, None);
}

// =============================================================================
// Parenthesis Display Tests
// =============================================================================

#[test]
fn test_paren_simple() {
    assert_expr_string("(1)", None);
    assert_expr_string("(metric)", None);
}

#[test]
fn test_paren_nested() {
    assert_expr_string("((1))", None);
    assert_expr_string("((1 + 2))", None);
}
