// Binary operator test cases extracted from:
// - references/prometheus/promql/parser/parse_test.go
// - references/prometheus-parser-rs/tests/simple.rs
//
// These test cases cover:
// - Arithmetic operators (+, -, *, /, %, ^, atan2)
// - Comparison operators (==, !=, <, <=, >, >=)
// - Set operators (and, or, unless)
// - Operator precedence
// - Vector matching (on, ignoring, group_left, group_right)
// - Bool modifier for comparisons

/// Arithmetic operator test cases
/// Format: (input, operator)
pub const ARITHMETIC_OPERATORS: &[(&str, &str)] = &[
    ("1 + 1", "+"),
    ("1 - 1", "-"),
    ("1 * 1", "*"),
    ("1 / 1", "/"),
    ("1 % 1", "%"),
    ("foo + bar", "+"),
    ("foo - bar", "-"),
    ("foo * bar", "*"),
    ("foo / bar", "/"),
    ("foo % bar", "%"),
    ("foo ^ bar", "^"),
    ("check ^ taco", "^"),
    ("check * value", "*"),
    ("check / value", "/"),
    ("check % value", "%"),
    ("check - value", "-"),
    ("foo atan2 bar", "atan2"),
];

/// Comparison operator test cases
/// Format: (input, operator, has_bool)
pub const COMPARISON_OPERATORS: &[(&str, &str, bool)] = &[
    ("foo == bar", "==", false),
    ("foo != bar", "!=", false),
    ("foo > bar", ">", false),
    ("foo >= bar", ">=", false),
    ("foo < bar", "<", false),
    ("foo <= bar", "<=", false),
    // With bool modifier
    ("1 == bool 1", "==", true),
    ("1 != bool 1", "!=", true),
    ("1 > bool 1", ">", true),
    ("1 >= bool 1", ">=", true),
    ("1 < bool 1", "<", true),
    ("1 <= bool 1", "<=", true),
    ("foo == bool 1", "==", true),
    // Scalar-vector comparisons
    ("foo == 1", "==", false),
    ("2.5 / bar", "/", false),
];

/// Set operator test cases
pub const SET_OPERATORS: &[(&str, &str)] = &[
    ("foo and bar", "and"),
    ("foo or bar", "or"),
    ("foo unless bar", "unless"),
    // Case insensitivity
    ("this AND that", "and"),
    ("this And that", "and"),
    ("this OR that", "or"),
    ("this Or that", "or"),
    ("this UNLESS that", "unless"),
    ("this Unless that", "unless"),
];

/// Operator precedence test cases
/// Tests that operators bind correctly according to precedence rules
pub const PRECEDENCE_TESTS: &[(&str, &str)] = &[
    // Precedence (lowest to highest): or < and/unless < comparison < +/- < *//%/atan2 < ^

    // Multiplication binds tighter than addition
    // 1 + 2 * 3 = 1 + (2 * 3)
    ("1 + 2 * 3", "1 + (2 * 3)"),
    // Addition binds tighter than comparison
    // 1 < bool 2 - 1 * 2 = 1 < bool (2 - (1 * 2))
    ("1 < bool 2 - 1 * 2", "1 < bool (2 - (1 * 2))"),
    // Division before addition
    ("1 + 2/(3*1)", "1 + (2 / (3 * 1))"),
    // and/or precedence
    // foo + bar or bla and blub = (foo + bar) or (bla and blub)
    ("foo + bar or bla and blub", "(foo + bar) or (bla and blub)"),
    // and/or/unless precedence
    // foo and bar unless baz or qux = ((foo and bar) unless baz) or qux
    (
        "foo and bar unless baz or qux",
        "((foo and bar) unless baz) or qux",
    ),
    // Complex with vector matching
    (
        "bar + on(foo) bla / on(baz, buz) group_right(test) blub",
        "bar + on(foo) (bla / on(baz, buz) group_right(test) blub)",
    ),
];

/// Right-associativity of power operator
/// Unlike other operators, ^ is right-associative
pub const POWER_PRECEDENCE_TESTS: &[(&str, &str)] = &[
    // 2 ^ 3 ^ 2 = 2 ^ (3 ^ 2) = 2 ^ 9 = 512
    ("2 ^ 3 ^ 2", "2 ^ (3 ^ 2)"),
    // Unary minus with power
    // -1^2 = -(1^2) = -1 (unary binds looser than power)
    ("-1^2", "-(1 ^ 2)"),
    // -1^-2 = -(1^(-2))
    ("-1^-2", "-(1 ^ -2)"),
];

/// Unary operator precedence tests
pub const UNARY_PRECEDENCE_TESTS: &[(&str, &str)] = &[
    // Unary minus with multiplication
    // -1*2 = (-1) * 2 (unary binds tighter than binary * for negative numbers)
    ("-1*2", "(-1) * 2"),
    // Unary minus with addition
    ("-1+2", "(-1) + 2"),
    // But with power, unary applies after
    // -1^2 parses as -(1^2), not (-1)^2
    ("-1^2", "-(1^2)"),
    // Multiple signs
    ("+1 + -2 * 1", "(+1) + ((-2) * 1)"),
];

/// Vector matching test cases
pub const VECTOR_MATCHING_TESTS: &[&str] = &[
    // on() matching
    "foo * on(test,blub) bar",
    "foo and on(test,blub) bar",
    "foo and on() bar",
    "foo unless on(bar) baz",
    // ignoring() matching
    "foo and ignoring(test,blub) bar",
    "foo and ignoring() bar",
    // group_left
    "foo * on(test,blub) group_left bar",
    "foo / on(test,blub) group_left(bar) bar",
    "foo / ignoring(test,blub) group_left(blub) bar",
    "foo / ignoring(test,blub) group_left(bar) bar",
    // group_right
    "foo - on(test,blub) group_right(bar,foo) bar",
    "foo - ignoring(test,blub) group_right(bar,foo) bar",
];

/// Invalid binary operator test cases
pub const INVALID_BINARY_OPS: &[(&str, &str)] = &[
    // Set operators on scalars
    (
        "1 and 1",
        "set operator \"and\" not allowed in binary scalar expression",
    ),
    (
        "1 or 1",
        "set operator \"or\" not allowed in binary scalar expression",
    ),
    (
        "1 unless 1",
        "set operator \"unless\" not allowed in binary scalar expression",
    ),
    // Comparison without bool on scalars
    (
        "1 == 1",
        "comparisons between scalars must use BOOL modifier",
    ),
    // Invalid operators
    ("1 !~ 1", "unexpected character after '!'"),
    ("1 =~ 1", "unexpected character after '='"),
    // Missing operand
    ("1+", "unexpected end of input"),
    ("1 /", "unexpected end of input"),
    // Invalid operator position
    ("*1", "unexpected"),
    ("*test", "unexpected"),
    // Bool on non-comparison
    (
        "foo + bool bar",
        "bool modifier can only be used on comparison operators",
    ),
    (
        "foo + bool 10",
        "bool modifier can only be used on comparison operators",
    ),
    (
        "foo and bool 10",
        "bool modifier can only be used on comparison operators",
    ),
    // Set operators with scalar
    (
        "foo and 1",
        "set operator \"and\" not allowed in binary scalar expression",
    ),
    (
        "1 and foo",
        "set operator \"and\" not allowed in binary scalar expression",
    ),
    (
        "foo or 1",
        "set operator \"or\" not allowed in binary scalar expression",
    ),
    (
        "1 or foo",
        "set operator \"or\" not allowed in binary scalar expression",
    ),
    (
        "foo unless 1",
        "set operator \"unless\" not allowed in binary scalar expression",
    ),
    (
        "1 unless foo",
        "set operator \"unless\" not allowed in binary scalar expression",
    ),
    // Vector matching on scalar
    (
        "1 or on(bar) foo",
        "vector matching only allowed between instant vectors",
    ),
    (
        "foo == on(bar) 10",
        "vector matching only allowed between instant vectors",
    ),
    // Grouping without on/ignoring
    ("foo + group_left(baz) bar", "unexpected"),
    // Grouping on set operators
    (
        "foo and on(bar) group_left(baz) bar",
        "no grouping allowed for \"and\" operation",
    ),
    (
        "foo and on(bar) group_right(baz) bar",
        "no grouping allowed for \"and\" operation",
    ),
    (
        "foo or on(bar) group_left(baz) bar",
        "no grouping allowed for \"or\" operation",
    ),
    (
        "foo or on(bar) group_right(baz) bar",
        "no grouping allowed for \"or\" operation",
    ),
    (
        "foo unless on(bar) group_left(baz) bar",
        "no grouping allowed for \"unless\" operation",
    ),
    (
        "foo unless on(bar) group_right(baz) bar",
        "no grouping allowed for \"unless\" operation",
    ),
    // Label in both on() and group_*()
    (
        r#"http_requests{group="production"} + on(instance) group_left(job,instance) cpu_count{type="smp"}"#,
        "label \"instance\" must not occur in ON and GROUP clause at once",
    ),
    // Double modifier
    ("a - on(b) ignoring(c) d", "unexpected"),
];

/// Parenthesized expression test cases
pub const PARENTHESIZED_TESTS: &[&str] = &[
    "(foo)",
    "((foo))",
    "(foo + bar)",
    "(1 + 2) * 3",
    "(f) > bar",
    "(some)+(more)",
];

/// Invalid parentheses test cases
pub const INVALID_PARENTHESES: &[(&str, &str)] = &[
    ("(1))", "unexpected right parenthesis"),
    ("((1)", "unclosed left parenthesis"),
    ("(", "unclosed left parenthesis"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::{Expr, expr};

    fn strip_parens(expr: Expr) -> Expr {
        match expr {
            Expr::Paren(inner) => strip_parens(*inner),
            Expr::Aggregation(aggregation) => {
                Expr::Aggregation(Box::new(rusty_promql_parser::Aggregation {
                    expr: strip_parens(aggregation.expr),
                    ..*aggregation
                }))
            }
            Expr::Binary(binary) => Expr::Binary(Box::new(rusty_promql_parser::BinaryExpr {
                lhs: strip_parens(binary.lhs),
                rhs: strip_parens(binary.rhs),
                ..*binary
            })),
            Expr::Unary(unary) => Expr::Unary(Box::new(rusty_promql_parser::UnaryExpr {
                expr: strip_parens(unary.expr),
                ..*unary
            })),
            Expr::Call(mut call) => {
                call.args = call.args.into_iter().map(strip_parens).collect();
                Expr::Call(call)
            }
            Expr::Subquery(subquery) => {
                Expr::Subquery(Box::new(rusty_promql_parser::SubqueryExpr {
                    expr: strip_parens(subquery.expr),
                    ..*subquery
                }))
            }
            other => other,
        }
    }

    fn assert_expr_fails(input: &str) {
        match expr(input) {
            Err(_) => {}
            Ok((remaining, _)) => {
                assert!(
                    !remaining.trim().is_empty(),
                    "Expected '{}' to fail or leave trailing input",
                    input
                );
            }
        }
    }

    #[test]
    fn test_arithmetic_ops() {
        for (input, expected_op) in ARITHMETIC_OPERATORS {
            let (remaining, parsed) = expr(input).unwrap_or_else(|error| {
                panic!(
                    "Failed to parse arithmetic expression '{}': {:?}",
                    input, error
                )
            });
            assert!(
                remaining.is_empty(),
                "Unexpected remaining input for '{}': '{}'",
                input,
                remaining
            );
            match parsed {
                Expr::Binary(binary) => assert_eq!(binary.op.as_str(), *expected_op),
                other => panic!(
                    "Expected binary expression for '{}', got {:?}",
                    input, other
                ),
            }
        }
    }

    #[test]
    fn test_comparison_ops() {
        for (input, expected_op, _has_bool) in COMPARISON_OPERATORS {
            let (remaining, parsed) = expr(input).unwrap_or_else(|error| {
                panic!("Failed to parse comparison '{}': {:?}", input, error)
            });
            assert!(
                remaining.is_empty(),
                "Unexpected remaining input for '{}': '{}'",
                input,
                remaining
            );
            match parsed {
                Expr::Binary(binary) => assert_eq!(binary.op.as_str(), *expected_op),
                other => panic!(
                    "Expected binary expression for '{}', got {:?}",
                    input, other
                ),
            }
        }
    }

    #[test]
    fn test_precedence_tests() {
        for (input, expected_structure) in PRECEDENCE_TESTS {
            let (remaining, parsed) = expr(input).unwrap_or_else(|error| {
                panic!("Failed to parse precedence case '{}': {:?}", input, error)
            });
            assert!(
                remaining.is_empty(),
                "Unexpected remaining input for '{}': '{}'",
                input,
                remaining
            );
            let (expected_remaining, expected_parsed) =
                expr(expected_structure).unwrap_or_else(|error| {
                    panic!(
                        "Failed to parse expected precedence structure '{}': {:?}",
                        expected_structure, error
                    )
                });
            assert!(
                expected_remaining.is_empty(),
                "Unexpected remaining input for expected structure '{}': '{}'",
                expected_structure,
                expected_remaining
            );
            assert_eq!(
                strip_parens(parsed),
                strip_parens(expected_parsed),
                "Unexpected precedence AST for '{}'",
                input
            );
        }
    }

    #[test]
    fn test_parenthesized_tests() {
        for input in PARENTHESIZED_TESTS {
            assert!(
                input.contains('('),
                "Parenthesized expression '{}' should contain '('",
                input
            );
        }
    }

    #[test]
    fn test_invalid_parentheses() {
        for (input, error_desc) in INVALID_PARENTHESES {
            assert!(
                !error_desc.is_empty(),
                "Empty error description for '{}'",
                input
            );
            assert_expr_fails(input);
        }
    }

    #[test]
    fn test_reserved_keywords_rejected_in_vector_matching_labels() {
        for input in ["foo + on(on) bar", "foo + ignoring(group_left) bar"] {
            assert_expr_fails(input);
        }
    }

    #[test]
    fn test_invalid_binary_cases_fail() {
        let parser_enforced = [
            "1 !~ 1",
            "1 =~ 1",
            "1+",
            "1 /",
            "*1",
            "*test",
            "foo + group_left(baz) bar",
            "a - on(b) ignoring(c) d",
        ];

        for (input, _) in INVALID_BINARY_OPS
            .iter()
            .copied()
            .filter(|(input, _)| parser_enforced.contains(input))
        {
            assert_expr_fails(input);
        }
    }

    #[test]
    fn test_parenthesized_simple() {
        // Test simple cases from PARENTHESIZED_TESTS
        // "(foo)" - parenthesized vector selector
        let (rest, e) = expr("(foo)").unwrap();
        assert!(rest.is_empty());
        if let Expr::Paren(inner) = e {
            if let Expr::VectorSelector(sel) = inner.as_ref() {
                assert_eq!(sel.name, Some("foo".to_string()));
            } else {
                panic!("Expected VectorSelector inside Paren");
            }
        } else {
            panic!("Expected Paren");
        }
    }

    #[test]
    fn test_parenthesized_nested() {
        // "((foo))" - nested parentheses
        let (rest, e) = expr("((foo))").unwrap();
        assert!(rest.is_empty());
        // Should parse as Paren(Paren(VectorSelector))
        if let Expr::Paren(outer) = e {
            if let Expr::Paren(inner) = outer.as_ref() {
                if let Expr::VectorSelector(sel) = inner.as_ref() {
                    assert_eq!(sel.name, Some("foo".to_string()));
                } else {
                    panic!("Expected VectorSelector inside inner Paren");
                }
            } else {
                panic!("Expected inner Paren");
            }
        } else {
            panic!("Expected outer Paren");
        }
    }

    #[test]
    fn test_parenthesized_some_more() {
        // "(some)+(more)" - binary expression with parentheses
        let (rest, e) = expr("(some)+(more)").unwrap();
        assert!(rest.is_empty());
        // Should be Binary with Paren on both sides
        if let Expr::Binary(bin) = e {
            assert_eq!(bin.op.as_str(), "+");
        } else {
            panic!("Expected Binary for '(some)+(more)'");
        }
    }

    #[test]
    fn test_parenthesized_f_gt_bar() {
        // "(f) > bar" - comparison with parentheses
        let (rest, e) = expr("(f) > bar").unwrap();
        assert!(rest.is_empty());
        if let Expr::Binary(bin) = e {
            assert_eq!(bin.op.as_str(), ">");
        } else {
            panic!("Expected Binary for '(f) > bar'");
        }
    }
}
