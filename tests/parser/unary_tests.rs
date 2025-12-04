// Unary operator test cases extracted from:
// - references/prometheus/promql/parser/parse_test.go
//
// These test cases cover:
// - Unary minus (negation)
// - Unary plus (no-op)
// - Operator precedence with unary operators

/// Valid unary minus (negation) test cases
pub const VALID_UNARY_MINUS: &[&str] = &[
    // Simple negation
    "-some_metric",
    "-1",
    "-1.5",
    "-0.5",
    // Negation of expressions
    "-(some_metric)",
    "-(some_metric + other_metric)",
    "-(rate(some_metric[5m]))",
    "-(sum(some_metric))",
    // Negation with selectors
    r#"-some_metric{job="foo"}"#,
    r#"-http_requests_total{method="GET"}"#,
    // Double negation
    "--some_metric",
    "- -some_metric",
    "- - -some_metric",
    // Negation in expressions
    "some_metric + -other_metric",
    "some_metric - -other_metric",
    "some_metric * -2",
    "-some_metric * -other_metric",
    // Negation with functions
    "-rate(some_metric[5m])",
    "-avg_over_time(some_metric[5m])",
    // Negation with aggregations
    "-sum(some_metric)",
    "-avg(some_metric) by (job)",
    "-topk(5, some_metric)",
];

/// Valid unary plus test cases
pub const VALID_UNARY_PLUS: &[&str] = &[
    // Simple positive (no-op)
    "+some_metric",
    "+1",
    "+1.5",
    // Positive of expressions
    "+(some_metric)",
    "+(some_metric + other_metric)",
    // With selectors
    r#"+some_metric{job="foo"}"#,
    // Double positive
    "++some_metric",
    "+ +some_metric",
    // Positive in expressions
    "some_metric + +other_metric",
    // Positive with functions
    "+rate(some_metric[5m])",
];

/// Mixed unary operators
pub const MIXED_UNARY_OPS: &[&str] = &[
    "+-some_metric",
    "-+some_metric",
    "+-+-some_metric",
    "+ -some_metric",
    "- +some_metric",
];

/// Unary operator precedence test cases
/// Format: (input, expected_parse_structure_description)
pub const UNARY_PRECEDENCE_TESTS: &[(&str, &str)] = &[
    // Unary minus has higher precedence than binary ops
    ("-a + b", "((-a) + b)"),
    ("a + -b", "(a + (-b))"),
    ("-a * b", "((-a) * b)"),
    ("a * -b", "(a * (-b))"),
    // Unary minus with power (special case - power binds tighter)
    ("-2^3", "(-(2^3))"), // NOT ((-2)^3)
    ("-a^b", "(-(a^b))"), // NOT ((-a)^b)
    // Multiple negations
    ("--a", "(-(-a))"),
    ("---a", "(-(-(-a)))"),
    // Unary with comparison
    ("-a > -b", "((-a) > (-b))"),
    ("-a == -b", "((-a) == (-b))"),
    // Unary in complex expressions
    ("-a + -b * -c", "((-a) + ((-b) * (-c)))"),
];

/// Invalid unary operator test cases
pub const INVALID_UNARY_OPS: &[(&str, &str)] = &[
    // Unary operator without operand
    ("-", "unexpected end of input"),
    ("+", "unexpected end of input"),
    // Unary operator before binary operator (incomplete expression)
    ("- +", "unexpected"),
    // Note: Most "invalid" unary operator cases would actually be
    // caught as general expression parsing errors
];

/// Unary operators with comments (edge cases)
pub const UNARY_WITH_WHITESPACE: &[&str] = &[
    "- some_metric",
    "-  some_metric",
    "-\tsome_metric",
    "-\nsome_metric",
    "+ some_metric",
    "+  some_metric",
];

/// Real-world expressions with unary operators
pub const REAL_WORLD_UNARY: &[&str] = &[
    // Negate a rate for display purposes
    "-rate(errors_total[5m])",
    // Calculate negative growth
    "-(rate(some_counter[5m]))",
    // Subtract a negated value
    "baseline - -adjustment",
    // Complex expression with negation
    "sum(rate(http_requests_total[5m])) * -1",
];

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::{Expr, UnaryOp, expr};

    #[test]
    fn test_unary_minus_parses() {
        for input in VALID_UNARY_MINUS {
            let result = expr(input);
            assert!(
                result.is_ok(),
                "Failed to parse unary minus expression '{}': {:?}",
                input,
                result.err()
            );
            let (remaining, _) = result.unwrap();
            assert!(
                remaining.is_empty(),
                "Unexpected remaining input after parsing '{}': '{}'",
                input,
                remaining
            );
        }
    }

    #[test]
    fn test_unary_plus_parses() {
        for input in VALID_UNARY_PLUS {
            let result = expr(input);
            assert!(
                result.is_ok(),
                "Failed to parse unary plus expression '{}': {:?}",
                input,
                result.err()
            );
            let (remaining, _) = result.unwrap();
            assert!(
                remaining.is_empty(),
                "Unexpected remaining input after parsing '{}': '{}'",
                input,
                remaining
            );
        }
    }

    #[test]
    fn test_mixed_unary_ops_parse() {
        for input in MIXED_UNARY_OPS {
            let result = expr(input);
            assert!(
                result.is_ok(),
                "Failed to parse mixed unary expression '{}': {:?}",
                input,
                result.err()
            );
        }
    }

    #[test]
    fn test_unary_with_whitespace_parses() {
        for input in UNARY_WITH_WHITESPACE {
            let result = expr(input);
            assert!(
                result.is_ok(),
                "Failed to parse unary with whitespace '{}': {:?}",
                input,
                result.err()
            );
        }
    }

    #[test]
    fn test_real_world_unary_parses() {
        for input in REAL_WORLD_UNARY {
            let result = expr(input);
            assert!(
                result.is_ok(),
                "Failed to parse real-world unary expression '{}': {:?}",
                input,
                result.err()
            );
        }
    }

    #[test]
    fn test_simple_unary_minus_structure() {
        let (_, e) = expr("-some_metric").unwrap();
        if let Expr::Unary(unary) = e {
            assert_eq!(unary.op, UnaryOp::Minus);
            assert!(matches!(unary.expr, Expr::VectorSelector(_)));
        } else {
            panic!("Expected Unary expression, got {:?}", e);
        }
    }

    #[test]
    fn test_simple_unary_plus_structure() {
        let (_, e) = expr("+some_metric").unwrap();
        if let Expr::Unary(unary) = e {
            assert_eq!(unary.op, UnaryOp::Plus);
            assert!(matches!(unary.expr, Expr::VectorSelector(_)));
        } else {
            panic!("Expected Unary expression, got {:?}", e);
        }
    }

    #[test]
    fn test_unary_minus_number() {
        let (_, e) = expr("-42").unwrap();
        // -42 could be parsed as Unary(Minus, Number(42)) or Number(-42)
        // depending on implementation. Either is valid.
        match e {
            Expr::Unary(unary) => {
                assert_eq!(unary.op, UnaryOp::Minus);
                if let Expr::Number(n) = unary.expr {
                    assert_eq!(n, 42.0);
                }
            }
            Expr::Number(n) => {
                assert_eq!(n, -42.0);
            }
            _ => panic!("Expected Unary or Number, got {:?}", e),
        }
    }

    #[test]
    fn test_double_negation_structure() {
        let (_, e) = expr("--some_metric").unwrap();
        if let Expr::Unary(outer) = e {
            assert_eq!(outer.op, UnaryOp::Minus);
            if let Expr::Unary(ref inner) = outer.expr {
                assert_eq!(inner.op, UnaryOp::Minus);
                assert!(matches!(inner.expr, Expr::VectorSelector(_)));
            } else {
                panic!("Expected inner Unary, got {:?}", outer.expr);
            }
        } else {
            panic!("Expected outer Unary, got {:?}", e);
        }
    }

    #[test]
    fn test_unary_in_binary_expr() {
        // "a + -b" should parse as Binary(a, +, Unary(-, b))
        let (_, e) = expr("a + -b").unwrap();
        if let Expr::Binary(bin) = e {
            assert!(matches!(bin.lhs, Expr::VectorSelector(_)));
            if let Expr::Unary(ref unary) = bin.rhs {
                assert_eq!(unary.op, UnaryOp::Minus);
                assert!(matches!(unary.expr, Expr::VectorSelector(_)));
            } else {
                panic!("Expected Unary on RHS, got {:?}", bin.rhs);
            }
        } else {
            panic!("Expected Binary expression, got {:?}", e);
        }
    }

    #[test]
    fn test_unary_display_roundtrip() {
        for input in VALID_UNARY_MINUS.iter().take(5) {
            let (_, e) = expr(input).unwrap();
            let displayed = e.to_string();
            let result = expr(&displayed);
            assert!(
                result.is_ok(),
                "Roundtrip failed for '{}' -> '{}': {:?}",
                input,
                displayed,
                result.err()
            );
        }
    }
}
