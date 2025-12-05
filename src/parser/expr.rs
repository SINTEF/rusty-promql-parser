//! Main expression parser for PromQL.
//!
//! This module provides the top-level [`expr`] function that parses any valid
//! PromQL expression into an AST. It uses a Pratt parser (precedence climbing)
//! algorithm for correct handling of binary operator precedence and associativity.
//!
//! # Expression Grammar (Simplified)
//!
//! ```text
//! expr          = unary_expr | binary_expr
//! binary_expr   = expr binary_op expr
//! unary_expr    = unary_op? postfix_expr
//! postfix_expr  = primary_expr postfix*
//! postfix       = subquery_range | matrix_range
//! primary_expr  = number | string | selector | paren_expr | function_call | aggregation
//! paren_expr    = "(" expr ")"
//! ```
//!
//! # Operator Precedence (Lowest to Highest)
//!
//! 1. `or` - Set union
//! 2. `and`, `unless` - Set intersection/difference
//! 3. `==`, `!=`, `<`, `<=`, `>`, `>=` - Comparison
//! 4. `+`, `-` - Addition/subtraction
//! 5. `*`, `/`, `%`, `atan2` - Multiplication/division
//! 6. `^` - Power (right-associative)
//!
//! # Examples
//!
//! ```rust
//! use rusty_promql_parser::parser::expr::expr;
//!
//! // Simple metric
//! let (rest, e) = expr("http_requests_total").unwrap();
//! assert!(rest.is_empty());
//!
//! // Binary expression with correct precedence
//! let (rest, e) = expr("1 + 2 * 3").unwrap(); // Parses as: 1 + (2 * 3)
//! assert!(rest.is_empty());
//!
//! // Complex aggregation
//! let (rest, e) = expr("sum(rate(http_requests[5m])) by (job)").unwrap();
//! assert!(rest.is_empty());
//! ```

use nom::{
    IResult, Parser,
    branch::alt,
    character::complete::char,
    combinator::{opt, peek},
    multi::separated_list0,
    sequence::{delimited, preceded, terminated},
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
    let (mut input, mut lhs) = parse_unary_expr(input)?;

    loop {
        // Try to parse: ws binary_op ws modifier? ws rhs
        let Ok((after_ws, _)) = ws_opt(input) else {
            break;
        };
        let Ok((after_op, op)) = binary_op(after_ws) else {
            break;
        };

        // Check precedence
        let op_precedence = op.precedence();
        if op_precedence < min_precedence {
            break;
        }

        // For right-associative operators (^), use same precedence for recursive call
        // For left-associative operators, use precedence + 1
        let next_min_precedence = if op.is_right_associative() {
            op_precedence
        } else {
            op_precedence + 1
        };

        // Parse: ws modifier? ws rhs
        let (remaining, (_, modifier, _, rhs)) = (ws_opt, opt(binary_modifier), ws_opt, |i| {
            parse_binary_expr(i, next_min_precedence)
        })
            .parse(after_op)?;

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
    let (mut rest, mut expr) = parse_primary_expr(input)?;

    // Try to parse subquery postfix operations
    // Use peek to check for subquery pattern without consuming input
    while (ws_opt, peek_subquery_start).parse(rest).is_ok() {
        let (remaining, (_, ((range, step), (at, offset)))) =
            (ws_opt, (subquery_range, parse_modifiers)).parse(rest)?;

        expr = Expr::Subquery(Box::new(SubqueryExpr {
            expr,
            range,
            step,
            offset,
            at,
        }));
        rest = remaining;
    }

    Ok((rest, expr))
}

/// Peek for subquery start pattern: `[duration:`
/// Helper with explicit return type for type inference
fn peek_subquery_start(input: &str) -> IResult<&str, ()> {
    if looks_like_subquery(input) {
        Ok((input, ()))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
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

    // Parse metric name followed by optional whitespace, then dispatch
    let (rest, (name, _)) = (metric_name, ws_opt).parse(input)?;

    // Use peek to check for '(' without consuming
    if peek_open_paren(rest).is_ok() {
        parse_function_call(rest, name)
    } else {
        parse_vector_selector_with_name(rest, name)
    }
}

/// Peek for opening parenthesis
/// Helper with explicit return type for type inference
fn peek_open_paren(input: &str) -> IResult<&str, char> {
    peek(char('(')).parse(input)
}

/// Peek for opening brace
/// Helper with explicit return type for type inference
fn peek_open_brace(input: &str) -> IResult<&str, char> {
    peek(char('{')).parse(input)
}

/// Parse an aggregation expression
fn parse_aggregation_expr(input: &str, op: Keyword) -> IResult<&str, Expr> {
    // Try to parse grouping before the expression
    let (rest, grouping_before) =
        preceded(ws_opt, opt(terminated(grouping, ws_opt))).parse(input)?;

    // Parse the arguments in parentheses
    let (rest, (param, inner_expr)) = delimited(
        (char('('), ws_opt),
        |i| {
            if op.is_aggregation_with_param() {
                // Parametric: parse parameter, comma, then inner expression
                let (rest, (param, _, _, _, inner)) =
                    (expr, ws_opt, char(','), ws_opt, expr).parse(i)?;
                Ok((rest, (Some(param), inner)))
            } else {
                // Non-parametric: just parse inner expression
                expr.map(|inner| (None, inner)).parse(i)
            }
        },
        (ws_opt, char(')')),
    )
    .parse(rest)?;

    // Try to parse grouping after the expression (if not already parsed)
    let (rest, grouping_after) = if grouping_before.is_none() {
        preceded(ws_opt, opt(grouping)).parse(rest)?
    } else {
        (rest, None)
    };

    let agg = Aggregation {
        op: op.as_str().to_string(),
        expr: inner_expr,
        param,
        grouping: grouping_before.or(grouping_after),
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

    // Parse optional label matchers (only if input starts with '{')
    // Using peek to check without copying/trimming
    let (rest, matchers) = if peek_open_brace(input).is_ok() {
        label_matchers(input)?
    } else {
        (input, Vec::new())
    };

    // Check if this is a matrix selector: ws + '[' but NOT subquery pattern
    if (ws_opt, peek_matrix_bracket).parse(rest).is_ok() {
        // Matrix selector: ws [duration] modifiers
        return (ws_opt, char('['), duration, char(']'), parse_modifiers)
            .map(|(_, _, range, _, (at, offset))| {
                let selector = VectorSelector {
                    name: Some(name.to_string()),
                    matchers: matchers.clone(),
                    offset,
                    at,
                };
                Expr::MatrixSelector(MatrixSelector { selector, range })
            })
            .parse(rest);
    }

    // Vector selector with optional modifiers
    (ws_opt, parse_modifiers)
        .map(|(_, (at, offset))| {
            let selector = VectorSelector {
                name: Some(name.to_string()),
                matchers: matchers.clone(),
                offset,
                at,
            };
            Expr::VectorSelector(selector)
        })
        .parse(rest)
}

/// Peek for matrix bracket: `[` but NOT subquery pattern `[duration:`
/// Helper with explicit return type for type inference
fn peek_matrix_bracket(input: &str) -> IResult<&str, char> {
    let (rest, c) = peek(char('[')).parse(input)?;
    // Make sure it's NOT a subquery
    if looks_like_subquery(input) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    Ok((rest, c))
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

    // Check if this is a matrix selector: ws + '[' but NOT subquery pattern
    if (ws_opt, peek_matrix_bracket).parse(rest).is_ok() {
        return (ws_opt, char('['), duration, char(']'), parse_modifiers)
            .map(|(_, _, range, _, (at, offset))| {
                let selector = VectorSelector {
                    name: name.clone(),
                    matchers: other_matchers.clone(),
                    offset,
                    at,
                };
                Expr::MatrixSelector(MatrixSelector { selector, range })
            })
            .parse(rest);
    }

    // Vector selector
    (ws_opt, parse_modifiers)
        .map(|(_, (at, offset))| {
            let selector = VectorSelector {
                name: name.clone(),
                matchers: other_matchers.clone(),
                offset,
                at,
            };
            Expr::VectorSelector(selector)
        })
        .parse(rest)
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
