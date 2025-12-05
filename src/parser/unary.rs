//! Unary expression parsing for PromQL.
//!
//! PromQL supports two unary operators:
//!
//! - `-expr` - Negation (multiplies by -1)
//! - `+expr` - Identity (no-op)
//!
//! Unary operators have higher precedence than binary operators,
//! except for the power operator `^`. This means `-2^3` is parsed as `-(2^3)`.
//!
//! # Examples
//!
//! ```rust
//! use rusty_promql_parser::parser::unary::unary_op;
//! use rusty_promql_parser::ast::UnaryOp;
//!
//! let (_, op) = unary_op("-").unwrap();
//! assert_eq!(op, UnaryOp::Minus);
//!
//! let (_, op) = unary_op("+").unwrap();
//! assert_eq!(op, UnaryOp::Plus);
//! ```
//!
//! # Common Usage
//!
//! ```text
//! -some_metric           # Negate a metric
//! -rate(requests[5m])    # Negate a function result
//! --metric               # Double negation (identity)
//! ```

use nom::{
    IResult, Parser,
    branch::alt,
    character::complete::{char, multispace0},
    combinator::{peek, value},
    sequence::preceded,
};

use crate::ast::UnaryOp;

/// Parse a unary operator
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::parser::unary::unary_op;
/// use rusty_promql_parser::ast::UnaryOp;
///
/// let (rest, op) = unary_op("-").unwrap();
/// assert_eq!(op, UnaryOp::Minus);
///
/// let (rest, op) = unary_op("+").unwrap();
/// assert_eq!(op, UnaryOp::Plus);
/// ```
pub fn unary_op(input: &str) -> IResult<&str, UnaryOp> {
    alt((
        value(UnaryOp::Minus, char('-')),
        value(UnaryOp::Plus, char('+')),
    ))
    .parse(input)
}

/// Check if the input starts with a unary operator (after optional whitespace).
///
/// This is useful for lookahead in the expression parser.
#[allow(dead_code)]
pub(crate) fn starts_with_unary(input: &str) -> bool {
    starts_with_unary_parser(input).is_ok()
}

// Helper parser with explicit return type for type inference
fn starts_with_unary_parser(input: &str) -> IResult<&str, char> {
    preceded(multispace0, peek(alt((char('-'), char('+'))))).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unary_op_minus() {
        let (rest, op) = unary_op("-").unwrap();
        assert!(rest.is_empty());
        assert_eq!(op, UnaryOp::Minus);
    }

    #[test]
    fn test_unary_op_plus() {
        let (rest, op) = unary_op("+").unwrap();
        assert!(rest.is_empty());
        assert_eq!(op, UnaryOp::Plus);
    }

    #[test]
    fn test_unary_op_with_remaining() {
        let (rest, op) = unary_op("-foo").unwrap();
        assert_eq!(rest, "foo");
        assert_eq!(op, UnaryOp::Minus);

        let (rest, op) = unary_op("+123").unwrap();
        assert_eq!(rest, "123");
        assert_eq!(op, UnaryOp::Plus);
    }

    #[test]
    fn test_unary_op_invalid() {
        assert!(unary_op("*").is_err());
        assert!(unary_op("/").is_err());
        assert!(unary_op("foo").is_err());
        assert!(unary_op("").is_err());
    }

    #[test]
    fn test_starts_with_unary() {
        assert!(starts_with_unary("-foo"));
        assert!(starts_with_unary("+foo"));
        assert!(starts_with_unary("  -foo"));
        assert!(starts_with_unary("  +foo"));
        assert!(!starts_with_unary("foo"));
        assert!(!starts_with_unary("*foo"));
        assert!(!starts_with_unary(""));
    }

    #[test]
    fn test_unary_op_display() {
        assert_eq!(UnaryOp::Minus.as_str(), "-");
        assert_eq!(UnaryOp::Plus.as_str(), "+");
        assert_eq!(UnaryOp::Minus.to_string(), "-");
        assert_eq!(UnaryOp::Plus.to_string(), "+");
    }
}
