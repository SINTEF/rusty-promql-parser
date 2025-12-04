//! Parse error tests for PromQL
//!
//! These tests verify that the parser correctly rejects invalid inputs.
//! We check that parsing fails, not the specific error message format
//! (since nom errors are low-level and don't include position ranges).

use rusty_promql_parser::parser::expr;

/// Helper to assert that parsing fails
fn assert_parse_fails(input: &str) {
    let result = expr(input);
    match result {
        Ok((rest, _)) if rest.trim().is_empty() => {
            panic!(
                "Expected parse error for '{}', but parsing succeeded",
                input
            );
        }
        Ok((_rest, _)) => {
            // Parsing partially succeeded but left unconsumed input - this is also a form of "failure"
            // for a complete parse
        }
        Err(_) => {
            // Expected
        }
    }
}

/// Helper to assert that parsing fails and check the result is Err
fn assert_parse_error(input: &str) {
    let result = expr(input);
    assert!(
        result.is_err(),
        "Expected parse error for '{}', but got: {:?}",
        input,
        result
    );
}

// =============================================================================
// Syntax Error Tests
// =============================================================================

#[test]
fn test_unclosed_brace() {
    assert_parse_error("foo{");
    assert_parse_error(r#"foo{bar="baz""#);
    assert_parse_error("{");
}

#[test]
fn test_unclosed_bracket() {
    assert_parse_error("foo[5m");
    assert_parse_error("foo[");
}

#[test]
fn test_unclosed_paren() {
    assert_parse_error("(foo");
    assert_parse_error("sum(foo");
    assert_parse_error("rate(foo[5m]");
}

#[test]
fn test_extra_closing_chars() {
    assert_parse_fails("foo}");
    assert_parse_fails("foo]");
    assert_parse_fails("foo)");
}

#[test]
fn test_incomplete_binary_expr() {
    assert_parse_error("1 +");
    assert_parse_error("foo -");
    // Note: "+ 1" is a valid unary expression (positive 1)
}

#[test]
fn test_incomplete_comparison() {
    assert_parse_error("foo ==");
    assert_parse_error("foo !=");
    assert_parse_error("foo >");
}

// =============================================================================
// Invalid Duration Tests
// =============================================================================

#[test]
fn test_invalid_duration_format() {
    // Number without unit
    assert_parse_error("foo[5]");
    // Invalid unit
    assert_parse_fails("foo[5x]");
    // Negative duration in brackets
    assert_parse_error("foo[-5m]");
}

#[test]
fn test_invalid_duration_in_offset() {
    // offset needs a duration
    assert_parse_fails("foo offset");
    assert_parse_fails("foo offset bar");
}

// =============================================================================
// Invalid Selector Tests
// =============================================================================

#[test]
fn test_invalid_label_matcher() {
    // Missing value
    assert_parse_error(r#"foo{bar=}"#);
    // Invalid operator
    assert_parse_fails(r#"foo{bar=="baz"}"#);
    // Missing operator
    assert_parse_error(r#"foo{bar "baz"}"#);
}

#[test]
fn test_invalid_label_name() {
    // Label name starting with number
    assert_parse_error(r#"foo{0bar="baz"}"#);
    // Empty label name
    assert_parse_error(r#"foo{=""}"#);
}

// =============================================================================
// Invalid Aggregation Tests
// =============================================================================

#[test]
fn test_aggregation_missing_parens() {
    assert_parse_fails("sum foo");
    assert_parse_fails("avg metric");
}

#[test]
fn test_aggregation_empty_parens() {
    // Empty aggregation is invalid
    assert_parse_error("sum()");
    assert_parse_error("avg()");
}

#[test]
fn test_parametric_aggregation_missing_param() {
    // topk needs a number parameter
    assert_parse_fails("topk(metric)");
    // quantile needs a number parameter
    assert_parse_fails("quantile(metric)");
}

// =============================================================================
// Invalid Function Tests
// =============================================================================

#[test]
fn test_function_missing_parens() {
    // Function name without parens - this is actually parsed as a metric name
    // so we check it produces a VectorSelector, not a Call
    let result = expr("rate");
    assert!(result.is_ok());
    let (_, e) = result.unwrap();
    // 'rate' without parens is just a metric name
    assert!(matches!(e, rusty_promql_parser::Expr::VectorSelector(_)));
}

// =============================================================================
// Invalid @ Modifier Tests
// =============================================================================

#[test]
fn test_at_without_timestamp() {
    // Parser leaves @ unconsumed when timestamp is missing
    assert_parse_fails("foo @");
    assert_parse_fails("foo @ bar");
}

#[test]
fn test_at_invalid_timestamp() {
    // Inf and NaN are rejected by at_modifier, so parser leaves them unconsumed
    assert_parse_fails("foo @ Inf");
    assert_parse_fails("foo @ -Inf");
    assert_parse_fails("foo @ NaN");
}

// =============================================================================
// Invalid Subquery Tests
// =============================================================================

#[test]
fn test_subquery_missing_range() {
    assert_parse_error("foo[:1m]");
    assert_parse_error("foo[:]");
}

#[test]
fn test_subquery_invalid_step() {
    // Negative step is invalid
    assert_parse_error("foo[5m:-1m]");
}

// =============================================================================
// Type Mismatch Tests (Semantic Errors)
// =============================================================================

// Note: These are semantic errors that a full PromQL implementation would catch,
// but our parser may accept them syntactically. We test that at least
// parsing doesn't crash.

#[test]
fn test_range_on_scalar_parses_but_invalid() {
    // These are syntactically parseable but semantically invalid
    // Parser may accept or reject depending on implementation
    let _ = expr("1[5m]");
    let _ = expr("3.14[5m]");
}

#[test]
fn test_range_on_string_parses_but_invalid() {
    let _ = expr(r#""hello"[5m]"#);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_input() {
    assert_parse_error("");
    assert_parse_error("   ");
}

#[test]
fn test_only_whitespace() {
    assert_parse_error("  \t\n  ");
}

#[test]
fn test_gibberish() {
    assert_parse_error("@#$%");
    assert_parse_error("!!!");
}
