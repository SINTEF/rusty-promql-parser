//! Main expression parser for PromQL
//!
//! This module provides the top-level expression parser that combines all
//! individual parsers into a unified grammar using a Pratt parser (precedence climbing)
//! for binary operators.
//!
//! Expression grammar (simplified):
//! ```text
//! expr          = unary_expr | binary_expr
//! binary_expr   = expr binary_op expr
//! unary_expr    = unary_op? postfix_expr
//! postfix_expr  = primary_expr postfix*
//! postfix       = subquery_range | matrix_range
//! primary_expr  = number | string | vector_selector | paren_expr | function_call | aggregation
//! paren_expr    = "(" expr ")"
//! ```

use nom::{
    IResult, Parser,
    branch::alt,
    character::complete::char,
    combinator::opt,
    multi::separated_list0,
    sequence::{delimited, preceded},
};

use crate::ast::{Aggregation, BinaryExpr, Call, Expr, SubqueryExpr, UnaryExpr};
use crate::lexer::{
    duration::duration,
    identifier::{Keyword, aggregation_op, metric_name},
    number::number,
    string::string_literal,
    whitespace::ws_opt,
};
use crate::parser::{
    aggregation::grouping,
    binary::{binary_modifier, binary_op},
    selector::{label_matchers, parse_modifiers},
    subquery::{looks_like_subquery, subquery_range},
    unary::unary_op,
};

/// Parse a PromQL expression
///
/// This is the main entry point for parsing PromQL expressions.
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::parser::expr::expr;
///
/// // Simple metric
/// let (rest, e) = expr("http_requests").unwrap();
/// assert!(rest.is_empty());
///
/// // Binary expression
/// let (rest, e) = expr("foo + bar").unwrap();
/// assert!(rest.is_empty());
///
/// // Complex expression
/// let (rest, e) = expr("sum(rate(http_requests[5m])) by (job)").unwrap();
/// assert!(rest.is_empty());
/// ```
pub fn expr(input: &str) -> IResult<&str, Expr> {
    // Skip leading whitespace, then parse binary expression with minimum precedence 0
    preceded(ws_opt, |i| parse_binary_expr(i, 0)).parse(input)
}

/// Parse a binary expression using Pratt parser (precedence climbing)
///
/// The `min_precedence` parameter ensures we only parse operators at or above
/// the given precedence level, which handles precedence correctly.
fn parse_binary_expr(input: &str, min_precedence: u8) -> IResult<&str, Expr> {
    // Parse the left-hand side (unary expression)
    let (mut input, mut lhs) = parse_unary_expr(input)?;

    loop {
        // Skip whitespace
        let (remaining, _) = ws_opt(input)?;

        // Try to parse a binary operator
        let op_result = binary_op(remaining);
        let (remaining, op) = match op_result {
            Ok((r, o)) => (r, o),
            Err(_) => break, // No operator, we're done
        };

        // Check precedence
        let op_precedence = op.precedence();
        if op_precedence < min_precedence {
            break; // Operator has lower precedence, stop
        }

        // For right-associative operators (^), use same precedence for recursive call
        // For left-associative operators, use precedence + 1
        let next_min_precedence = if op.is_right_associative() {
            op_precedence
        } else {
            op_precedence + 1
        };

        // Skip whitespace after operator
        let (remaining, _) = ws_opt(remaining)?;

        // Try to parse an optional modifier (bool, on, ignoring, etc.)
        let (remaining, modifier) = opt(binary_modifier).parse(remaining)?;

        // Skip whitespace after modifier
        let (remaining, _) = ws_opt(remaining)?;

        // Parse the right-hand side with the appropriate precedence
        let (remaining, rhs) = parse_binary_expr(remaining, next_min_precedence)?;

        // Build the binary expression
        lhs = Expr::Binary(Box::new(BinaryExpr {
            op,
            lhs,
            rhs,
            modifier,
        }));
        input = remaining;
    }

    Ok((input, lhs))
}

/// Parse a unary expression: `unary_op? postfix_expr`
fn parse_unary_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        // Unary operator followed by another unary expression (recursive)
        // This handles chained unary operators like `--foo` or `-+foo`
        // Note: -2^3 = -(2^3), not (-2)^3, because unary binds looser than ^
        (unary_op, ws_opt, parse_unary_expr)
            .map(|(op, _, operand)| Expr::Unary(Box::new(UnaryExpr { op, expr: operand }))),
        // No unary operator, fall through to postfix
        parse_postfix_expr,
    ))
    .parse(input)
}

/// Parse a postfix expression: `primary_expr postfix*`
///
/// Postfix operations include:
/// - Subquery: `[5m:1m]`
/// - Modifiers: `offset 5m`, `@ start()`
fn parse_postfix_expr(input: &str) -> IResult<&str, Expr> {
    let (mut input, mut expr) = parse_primary_expr(input)?;

    // Try to parse postfix operations
    loop {
        let (remaining, _) = ws_opt(input)?;

        // Check for subquery bracket
        if remaining.starts_with('[') && looks_like_subquery(remaining) {
            let (remaining, (range, step)) = subquery_range(remaining)?;
            let (remaining, (at, offset)) = parse_modifiers(remaining)?;

            expr = Expr::Subquery(Box::new(SubqueryExpr {
                expr,
                range,
                step,
                offset,
                at,
            }));
            input = remaining;
            continue;
        }

        // No more postfix operations
        break;
    }

    Ok((input, expr))
}

/// Parse a primary expression (atoms)
fn parse_primary_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        // Parenthesized expression
        parse_paren_expr,
        // Number literal (must come before identifier to handle negative numbers correctly)
        parse_number_literal,
        // String literal
        parse_string_literal,
        // Selector starting with { (labels only, no metric name prefix)
        parse_labels_only_selector,
        // Aggregation, function call, or vector selector
        // (these all start with an identifier, so we handle them together)
        parse_identifier_expr,
    ))
    .parse(input)
}

/// Parse a parenthesized expression: `( expr )`
fn parse_paren_expr(input: &str) -> IResult<&str, Expr> {
    delimited((char('('), ws_opt), expr, (ws_opt, char(')')))
        .map(|inner| Expr::Paren(Box::new(inner)))
        .parse(input)
}

/// Parse a number literal
fn parse_number_literal(input: &str) -> IResult<&str, Expr> {
    number.map(Expr::Number).parse(input)
}

/// Parse a string literal
fn parse_string_literal(input: &str) -> IResult<&str, Expr> {
    string_literal.map(Expr::String).parse(input)
}

/// Parse an expression starting with an identifier
///
/// This handles:
/// - Aggregation operators: `sum(...)`, `avg by (...) (...)`
/// - Function calls: `rate(...)`, `abs(...)`
/// - Vector selectors: `metric`, `metric{labels}`
fn parse_identifier_expr(input: &str) -> IResult<&str, Expr> {
    // First, check if this is an aggregation operator
    if let Ok((rest, op)) = aggregation_op(input) {
        return parse_aggregation_expr(rest, op);
    }

    // Try to parse as metric name (for vector selector)
    let (rest, name) = metric_name(input)?;

    // Check what follows
    let (rest, _) = ws_opt(rest)?;

    // If followed by `(`, it's a function call
    if rest.starts_with('(') {
        return parse_function_call(rest, name);
    }

    // Otherwise it's a vector selector (possibly with labels and modifiers)
    parse_vector_selector_with_name(rest, name)
}

/// Parse an aggregation expression
fn parse_aggregation_expr(input: &str, op: Keyword) -> IResult<&str, Expr> {
    let (rest, _) = ws_opt(input)?;

    // Try to parse grouping before the expression
    let (rest, grouping_before) = opt((grouping, ws_opt).map(|(g, _)| g)).parse(rest)?;

    // Parse the arguments in parentheses
    let (rest, _) = char('(')(rest)?;
    let (rest, _) = ws_opt(rest)?;

    // For parametric aggregations (topk, quantile, etc.), first arg is the parameter
    let (rest, (param, inner_expr)) = if op.is_aggregation_with_param() {
        // Parse parameter expression
        let (rest, param) = expr(rest)?;
        let (rest, _) = ws_opt(rest)?;
        let (rest, _) = char(',')(rest)?;
        let (rest, _) = ws_opt(rest)?;
        // Parse inner expression
        let (rest, inner) = expr(rest)?;
        (rest, (Some(param), inner))
    } else {
        // Just parse inner expression
        let (rest, inner) = expr(rest)?;
        (rest, (None, inner))
    };

    let (rest, _) = ws_opt(rest)?;
    let (rest, _) = char(')')(rest)?;
    let (rest, _) = ws_opt(rest)?;

    // Try to parse grouping after the expression (if not already parsed)
    let (rest, grouping_after) = if grouping_before.is_none() {
        opt(grouping).parse(rest)?
    } else {
        (rest, None)
    };

    let grouping = grouping_before.or(grouping_after);

    let agg = Aggregation {
        op: op.as_str().to_string(),
        expr: inner_expr,
        param,
        grouping,
    };

    Ok((rest, Expr::Aggregation(Box::new(agg))))
}

/// Parse a function call
fn parse_function_call<'a>(input: &'a str, name: &str) -> IResult<&'a str, Expr> {
    delimited(
        (char('('), ws_opt),
        separated_list0((ws_opt, char(','), ws_opt), expr),
        (ws_opt, opt((char(','), ws_opt)), char(')')),
    )
    .map(|args| Expr::Call(Call::new(name, args)))
    .parse(input)
}

/// Parse a vector selector starting with a known metric name
fn parse_vector_selector_with_name<'a>(input: &'a str, name: &str) -> IResult<&'a str, Expr> {
    use crate::parser::selector::{MatrixSelector, VectorSelector};

    let mut rest = input;
    let mut matchers = Vec::new();

    // Check for label matchers
    if rest.starts_with('{') {
        let (remaining, labels) = label_matchers(rest)?;
        matchers = labels;
        rest = remaining;
    }

    let (rest, _) = ws_opt(rest)?;

    // Check for matrix selector bracket
    if rest.starts_with('[') && !looks_like_subquery(rest) {
        // Matrix selector
        let (remaining, _) = char('[')(rest)?;
        let (remaining, range) = duration(remaining)?;
        let (remaining, _) = char(']')(remaining)?;

        let (remaining, (at, offset)) = parse_modifiers(remaining)?;

        let selector = VectorSelector {
            name: Some(name.to_string()),
            matchers,
            offset,
            at,
        };

        return Ok((
            remaining,
            Expr::MatrixSelector(MatrixSelector { selector, range }),
        ));
    }

    // Vector selector with optional modifiers
    let (rest, (at, offset)) = parse_modifiers(rest)?;

    let selector = VectorSelector {
        name: Some(name.to_string()),
        matchers,
        offset,
        at,
    };

    Ok((rest, Expr::VectorSelector(selector)))
}

/// Parse a vector selector starting with just labels (no metric name)
fn parse_labels_only_selector(input: &str) -> IResult<&str, Expr> {
    use crate::parser::selector::{LabelMatchOp, MatrixSelector, VectorSelector};

    let (rest, matchers) = label_matchers(input)?;

    // Extract __name__ matcher if present
    let name = matchers
        .iter()
        .find(|m| m.name == "__name__" && m.op == LabelMatchOp::Equal)
        .map(|m| m.value.clone());

    // Filter out the __name__= matcher that we're using as the name
    let other_matchers: Vec<_> = if name.is_some() {
        matchers
            .into_iter()
            .filter(|m| !(m.name == "__name__" && m.op == LabelMatchOp::Equal))
            .collect()
    } else {
        matchers
    };

    let (rest, _) = ws_opt(rest)?;

    // Check for matrix selector bracket
    if rest.starts_with('[') && !looks_like_subquery(rest) {
        let (remaining, _) = char('[')(rest)?;
        let (remaining, range) = duration(remaining)?;
        let (remaining, _) = char(']')(remaining)?;

        let (remaining, (at, offset)) = parse_modifiers(remaining)?;

        let selector = VectorSelector {
            name,
            matchers: other_matchers,
            offset,
            at,
        };

        return Ok((
            remaining,
            Expr::MatrixSelector(MatrixSelector { selector, range }),
        ));
    }

    // Vector selector
    let (rest, (at, offset)) = parse_modifiers(rest)?;

    let selector = VectorSelector {
        name,
        matchers: other_matchers,
        offset,
        at,
    };

    Ok((rest, Expr::VectorSelector(selector)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, UnaryOp};

    #[test]
    fn test_parse_number() {
        let (rest, e) = expr("42").unwrap();
        assert!(rest.is_empty());
        assert_eq!(e, Expr::Number(42.0));
    }

    #[test]
    fn test_parse_string() {
        let (rest, e) = expr(r#""hello""#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(e, Expr::String("hello".to_string()));
    }

    #[test]
    fn test_parse_vector_selector() {
        let (rest, e) = expr("http_requests").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::VectorSelector(v) => {
                assert_eq!(v.name, Some("http_requests".to_string()));
            }
            _ => panic!("Expected VectorSelector"),
        }
    }

    #[test]
    fn test_parse_vector_selector_with_labels() {
        let (rest, e) = expr(r#"http_requests{job="api"}"#).unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::VectorSelector(v) => {
                assert_eq!(v.name, Some("http_requests".to_string()));
                assert_eq!(v.matchers.len(), 1);
                assert_eq!(v.matchers[0].name, "job");
                assert_eq!(v.matchers[0].value, "api");
            }
            _ => panic!("Expected VectorSelector"),
        }
    }

    #[test]
    fn test_parse_matrix_selector() {
        let (rest, e) = expr("http_requests[5m]").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::MatrixSelector(m) => {
                assert_eq!(m.selector.name, Some("http_requests".to_string()));
                assert_eq!(m.range.as_millis(), 5 * 60 * 1000);
            }
            _ => panic!("Expected MatrixSelector"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let (rest, e) = expr("rate(http_requests[5m])").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Call(c) => {
                assert_eq!(c.name, "rate");
                assert_eq!(c.args.len(), 1);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_parse_aggregation() {
        let (rest, e) = expr("sum(metric)").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Aggregation(a) => {
                assert_eq!(a.op, "sum");
            }
            _ => panic!("Expected Aggregation"),
        }
    }

    #[test]
    fn test_parse_aggregation_with_grouping() {
        let (rest, e) = expr("sum by (job) (metric)").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Aggregation(a) => {
                assert_eq!(a.op, "sum");
                assert!(a.grouping.is_some());
            }
            _ => panic!("Expected Aggregation"),
        }
    }

    #[test]
    fn test_parse_binary_add() {
        let (rest, e) = expr("1 + 2").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Add);
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_parse_binary_precedence() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let (rest, e) = expr("1 + 2 * 3").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Add);
                match b.rhs {
                    Expr::Binary(inner) => {
                        assert_eq!(inner.op, BinaryOp::Mul);
                    }
                    _ => panic!("Expected inner Binary"),
                }
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_parse_binary_right_associative() {
        // 2 ^ 3 ^ 2 should parse as 2 ^ (3 ^ 2)
        let (rest, e) = expr("2 ^ 3 ^ 2").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Pow);
                assert_eq!(b.lhs, Expr::Number(2.0));
                match b.rhs {
                    Expr::Binary(inner) => {
                        assert_eq!(inner.op, BinaryOp::Pow);
                        assert_eq!(inner.lhs, Expr::Number(3.0));
                        assert_eq!(inner.rhs, Expr::Number(2.0));
                    }
                    _ => panic!("Expected inner Binary"),
                }
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_parse_unary_minus() {
        let (rest, e) = expr("-42").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Unary(u) => {
                assert_eq!(u.op, UnaryOp::Minus);
                assert_eq!(u.expr, Expr::Number(42.0));
            }
            _ => panic!("Expected Unary"),
        }
    }

    #[test]
    fn test_parse_unary_with_binary() {
        // -1 + 2 should parse as (-1) + 2
        let (rest, e) = expr("-1 + 2").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Add);
                match b.lhs {
                    Expr::Unary(u) => {
                        assert_eq!(u.op, UnaryOp::Minus);
                    }
                    _ => panic!("Expected Unary as lhs"),
                }
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_parse_paren() {
        let (rest, e) = expr("(1 + 2)").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Paren(inner) => match *inner {
                Expr::Binary(b) => {
                    assert_eq!(b.op, BinaryOp::Add);
                }
                _ => panic!("Expected Binary inside Paren"),
            },
            _ => panic!("Expected Paren"),
        }
    }

    #[test]
    fn test_parse_paren_affects_precedence() {
        // (1 + 2) * 3 should parse differently than 1 + 2 * 3
        let (rest, e) = expr("(1 + 2) * 3").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Mul);
                match b.lhs {
                    Expr::Paren(inner) => match *inner {
                        Expr::Binary(b) => {
                            assert_eq!(b.op, BinaryOp::Add);
                        }
                        _ => panic!("Expected Binary inside Paren"),
                    },
                    _ => panic!("Expected Paren as lhs"),
                }
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_parse_subquery() {
        let (rest, e) = expr("metric[5m:1m]").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Subquery(s) => {
                assert_eq!(s.range.as_millis(), 5 * 60 * 1000);
                assert_eq!(s.step.unwrap().as_millis(), 60 * 1000);
            }
            _ => panic!("Expected Subquery"),
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let (rest, e) = expr("sum(rate(http_requests[5m])) by (job)").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Aggregation(a) => {
                assert_eq!(a.op, "sum");
                match a.expr {
                    Expr::Call(c) => {
                        assert_eq!(c.name, "rate");
                    }
                    _ => panic!("Expected Call inside Aggregation"),
                }
            }
            _ => panic!("Expected Aggregation"),
        }
    }

    #[test]
    fn test_parse_binary_with_modifier() {
        let (rest, e) = expr("foo + on(job) bar").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Add);
                assert!(b.modifier.is_some());
                let m = b.modifier.unwrap();
                assert!(m.matching.is_some());
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_parse_binary_bool() {
        let (rest, e) = expr("foo == bool bar").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Eq);
                assert!(b.modifier.is_some());
                let m = b.modifier.unwrap();
                assert!(m.return_bool);
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_parse_set_operators() {
        for (input, expected_op) in [
            ("foo and bar", BinaryOp::And),
            ("foo or bar", BinaryOp::Or),
            ("foo unless bar", BinaryOp::Unless),
        ] {
            let (rest, e) = expr(input).unwrap();
            assert!(rest.is_empty(), "Failed for: {}", input);
            match e {
                Expr::Binary(b) => {
                    assert_eq!(b.op, expected_op, "Failed for: {}", input);
                }
                _ => panic!("Expected Binary for: {}", input),
            }
        }
    }

    #[test]
    fn test_parse_offset_modifier() {
        let (rest, e) = expr("foo offset 5m").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::VectorSelector(v) => {
                assert!(v.offset.is_some());
                assert_eq!(v.offset.unwrap().as_millis(), 5 * 60 * 1000);
            }
            _ => panic!("Expected VectorSelector"),
        }
    }

    #[test]
    fn test_parse_at_modifier() {
        let (rest, e) = expr("foo @ 1609459200").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::VectorSelector(v) => {
                assert!(v.at.is_some());
            }
            _ => panic!("Expected VectorSelector"),
        }
    }

    #[test]
    fn test_parse_topk() {
        let (rest, e) = expr("topk(5, metric)").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Aggregation(a) => {
                assert_eq!(a.op, "topk");
                assert!(a.param.is_some());
                assert_eq!(*a.param.as_ref().unwrap(), Expr::Number(5.0));
            }
            _ => panic!("Expected Aggregation"),
        }
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let (rest, e) = expr("  foo   +   bar  ").unwrap();
        assert_eq!(rest.trim(), "");
        match e {
            Expr::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Add);
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_parse_subquery_with_both_modifiers() {
        // Test @ before offset
        let (rest, e) = expr("some_metric[5m:1m] @ 1609459200 offset 10m").unwrap();
        assert!(rest.is_empty());
        match e {
            Expr::Subquery(s) => {
                assert!(s.at.is_some(), "@ modifier should be present");
                assert!(s.offset.is_some(), "offset modifier should be present");
            }
            _ => panic!("Expected Subquery"),
        }

        // Test offset before @ - this order should also work
        let (rest, e) = expr("some_metric[5m:1m] offset 10m @ 1609459200").unwrap();
        assert!(
            rest.is_empty(),
            "Parser did not consume entire input, remaining: '{}'",
            rest
        );
        match e {
            Expr::Subquery(s) => {
                assert!(s.at.is_some(), "@ modifier should be present");
                assert!(s.offset.is_some(), "offset modifier should be present");
            }
            _ => panic!("Expected Subquery"),
        }
    }
}
