//! Binary expression parsing for PromQL.
//!
//! This module handles parsing of binary operators and their modifiers.
//!
//! # Binary Operators
//!
//! Operators listed from lowest to highest precedence:
//!
//! | Precedence | Operators                  | Description              |
//! |------------|----------------------------|--------------------------|
//! | 1          | `or`                       | Set union                |
//! | 2          | `and`, `unless`            | Set intersection/diff    |
//! | 3          | `==`, `!=`, `<`, `<=`, `>`, `>=` | Comparison         |
//! | 4          | `+`, `-`                   | Addition/subtraction     |
//! | 5          | `*`, `/`, `%`, `atan2`     | Multiplication/division  |
//! | 6          | `^`                        | Power (right-associative)|
//!
//! # Vector Matching Modifiers
//!
//! Binary operations between vectors can use matching modifiers:
//!
//! - `on(label, ...)` - Match only on specified labels
//! - `ignoring(label, ...)` - Match ignoring specified labels
//! - `group_left(label, ...)` - Many-to-one matching
//! - `group_right(label, ...)` - One-to-many matching
//! - `bool` - Return 0/1 instead of filtering (for comparisons)
//!
//! # Examples
//!
//! ```rust
//! use rusty_promql_parser::parser::binary::binary_op;
//! use rusty_promql_parser::ast::BinaryOp;
//!
//! let (_, op) = binary_op("+").unwrap();
//! assert_eq!(op, BinaryOp::Add);
//!
//! let (_, op) = binary_op("and").unwrap();
//! assert_eq!(op, BinaryOp::And);
//! ```

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{char, satisfy},
    combinator::{map, not, opt, peek, value},
    multi::separated_list0,
    sequence::delimited,
};

use crate::ast::{
    BinaryModifier, BinaryOp, GroupModifier, GroupSide, VectorMatching, VectorMatchingOp,
};
use crate::lexer::{identifier::label_name, whitespace::ws_opt};

/// Parser that succeeds only at a word boundary (not followed by alphanumeric or underscore)
fn word_boundary(input: &str) -> IResult<&str, ()> {
    not(peek(satisfy(|c| c.is_alphanumeric() || c == '_'))).parse(input)
}

/// Parse a binary operator
///
/// Handles all binary operators including the keyword operators
/// (and, or, unless, atan2).
pub fn binary_op(input: &str) -> IResult<&str, BinaryOp> {
    alt((
        // Two-character operators must come before single-character
        value(BinaryOp::Eq, tag("==")),
        value(BinaryOp::Ne, tag("!=")),
        value(BinaryOp::Le, tag("<=")),
        value(BinaryOp::Ge, tag(">=")),
        // Single-character operators
        value(BinaryOp::Add, tag("+")),
        value(BinaryOp::Sub, tag("-")),
        value(BinaryOp::Mul, tag("*")),
        value(BinaryOp::Div, tag("/")),
        value(BinaryOp::Mod, tag("%")),
        value(BinaryOp::Pow, tag("^")),
        value(BinaryOp::Lt, tag("<")),
        value(BinaryOp::Gt, tag(">")),
        // Keyword operators (case-insensitive)
        keyword_binary_op,
    ))
    .parse(input)
}

/// Parse keyword binary operators (case-insensitive)
fn keyword_binary_op(input: &str) -> IResult<&str, BinaryOp> {
    // We need to ensure these are complete words, not prefixes
    (
        alt((
            value(BinaryOp::And, tag_no_case("and")),
            value(BinaryOp::Or, tag_no_case("or")),
            value(BinaryOp::Unless, tag_no_case("unless")),
            value(BinaryOp::Atan2, tag_no_case("atan2")),
        )),
        word_boundary,
    )
        .map(|(op, _)| op)
        .parse(input)
}

/// Parse the `bool` modifier
fn bool_modifier(input: &str) -> IResult<&str, bool> {
    (tag_no_case("bool"), word_boundary)
        .map(|_| true)
        .parse(input)
}

/// Parse the matching operation (on/ignoring)
fn vector_matching_op(input: &str) -> IResult<&str, VectorMatchingOp> {
    (
        alt((
            value(VectorMatchingOp::On, tag_no_case("on")),
            value(VectorMatchingOp::Ignoring, tag_no_case("ignoring")),
        )),
        word_boundary,
    )
        .map(|(op, _)| op)
        .parse(input)
}

/// Parse a label list in parentheses: `(label1, label2)`
fn label_list(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        (char('('), ws_opt),
        separated_list0(
            delimited(ws_opt, char(','), ws_opt),
            map(label_name, |s| s.to_string()),
        ),
        (ws_opt, char(')')),
    )
    .parse(input)
}

/// Parse the group modifier (group_left/group_right)
fn group_modifier(input: &str) -> IResult<&str, GroupModifier> {
    (
        alt((
            value(GroupSide::Left, tag_no_case("group_left")),
            value(GroupSide::Right, tag_no_case("group_right")),
        )),
        word_boundary,
        ws_opt,
        opt(label_list),
    )
        .map(|(side, _, _, labels)| GroupModifier {
            side,
            labels: labels.unwrap_or_default(),
        })
        .parse(input)
}

/// Parse vector matching specification: `on(labels) group_left(labels)`
fn vector_matching(input: &str) -> IResult<&str, VectorMatching> {
    (
        vector_matching_op,
        ws_opt,
        label_list,
        ws_opt,
        opt(group_modifier),
    )
        .map(|(op, _, labels, _, group)| VectorMatching { op, labels, group })
        .parse(input)
}

/// Parse binary expression modifier: `bool on(labels) group_left(labels)`
///
/// This parses the optional modifiers that can appear between the operator
/// and the right-hand side operand.
pub(crate) fn binary_modifier(input: &str) -> IResult<&str, BinaryModifier> {
    let (rest, (_, return_bool, _, matching)) =
        (ws_opt, opt(bool_modifier), ws_opt, opt(vector_matching)).parse(input)?;

    // If neither bool nor matching, fail
    if return_bool.is_none() && matching.is_none() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    Ok((
        rest,
        BinaryModifier {
            return_bool: return_bool.unwrap_or(false),
            matching,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Binary operator tests
    #[test]
    fn test_binary_op_arithmetic() {
        assert_eq!(binary_op("+").unwrap().1, BinaryOp::Add);
        assert_eq!(binary_op("-").unwrap().1, BinaryOp::Sub);
        assert_eq!(binary_op("*").unwrap().1, BinaryOp::Mul);
        assert_eq!(binary_op("/").unwrap().1, BinaryOp::Div);
        assert_eq!(binary_op("%").unwrap().1, BinaryOp::Mod);
        assert_eq!(binary_op("^").unwrap().1, BinaryOp::Pow);
    }

    #[test]
    fn test_binary_op_comparison() {
        assert_eq!(binary_op("==").unwrap().1, BinaryOp::Eq);
        assert_eq!(binary_op("!=").unwrap().1, BinaryOp::Ne);
        assert_eq!(binary_op("<").unwrap().1, BinaryOp::Lt);
        assert_eq!(binary_op("<=").unwrap().1, BinaryOp::Le);
        assert_eq!(binary_op(">").unwrap().1, BinaryOp::Gt);
        assert_eq!(binary_op(">=").unwrap().1, BinaryOp::Ge);
    }

    #[test]
    fn test_binary_op_keywords() {
        assert_eq!(binary_op("and").unwrap().1, BinaryOp::And);
        assert_eq!(binary_op("AND").unwrap().1, BinaryOp::And);
        assert_eq!(binary_op("or").unwrap().1, BinaryOp::Or);
        assert_eq!(binary_op("OR").unwrap().1, BinaryOp::Or);
        assert_eq!(binary_op("unless").unwrap().1, BinaryOp::Unless);
        assert_eq!(binary_op("UNLESS").unwrap().1, BinaryOp::Unless);
        assert_eq!(binary_op("atan2").unwrap().1, BinaryOp::Atan2);
        assert_eq!(binary_op("ATAN2").unwrap().1, BinaryOp::Atan2);
    }

    #[test]
    fn test_binary_op_word_boundary() {
        // "andy" should not match "and"
        assert!(binary_op("andy").is_err());
        // "orange" should not match "or"
        assert!(binary_op("orange").is_err());
        // "atan2x" should not match "atan2"
        assert!(binary_op("atan2x").is_err());
    }

    #[test]
    fn test_binary_op_with_remaining() {
        let (rest, op) = binary_op("+ foo").unwrap();
        assert_eq!(op, BinaryOp::Add);
        assert_eq!(rest, " foo");

        let (rest, op) = binary_op("and bar").unwrap();
        assert_eq!(op, BinaryOp::And);
        assert_eq!(rest, " bar");
    }

    // Vector matching tests
    #[test]
    fn test_vector_matching_on() {
        let (rest, vm) = vector_matching("on(job, instance)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(vm.op, VectorMatchingOp::On);
        assert_eq!(vm.labels, vec!["job", "instance"]);
        assert!(vm.group.is_none());
    }

    #[test]
    fn test_vector_matching_ignoring() {
        let (rest, vm) = vector_matching("ignoring(instance)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(vm.op, VectorMatchingOp::Ignoring);
        assert_eq!(vm.labels, vec!["instance"]);
    }

    #[test]
    fn test_vector_matching_empty() {
        let (rest, vm) = vector_matching("on()").unwrap();
        assert!(rest.is_empty());
        assert_eq!(vm.op, VectorMatchingOp::On);
        assert!(vm.labels.is_empty());
    }

    #[test]
    fn test_vector_matching_with_group_left() {
        let (rest, vm) = vector_matching("on(job) group_left").unwrap();
        assert!(rest.is_empty());
        assert_eq!(vm.op, VectorMatchingOp::On);
        let group = vm.group.unwrap();
        assert_eq!(group.side, GroupSide::Left);
        assert!(group.labels.is_empty());
    }

    #[test]
    fn test_vector_matching_with_group_right_labels() {
        let (rest, vm) = vector_matching("ignoring(instance) group_right(job)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(vm.op, VectorMatchingOp::Ignoring);
        let group = vm.group.unwrap();
        assert_eq!(group.side, GroupSide::Right);
        assert_eq!(group.labels, vec!["job"]);
    }

    #[test]
    fn test_vector_matching_case_insensitive() {
        let (_, vm) = vector_matching("ON(job)").unwrap();
        assert_eq!(vm.op, VectorMatchingOp::On);

        let (_, vm) = vector_matching("IGNORING(job)").unwrap();
        assert_eq!(vm.op, VectorMatchingOp::Ignoring);

        let (_, vm) = vector_matching("on(job) GROUP_LEFT").unwrap();
        assert!(vm.group.is_some());
    }

    // Binary modifier tests
    #[test]
    fn test_binary_modifier_bool_only() {
        let (rest, m) = binary_modifier(" bool").unwrap();
        assert!(rest.is_empty() || rest.chars().all(|c| c.is_whitespace()));
        assert!(m.return_bool);
        assert!(m.matching.is_none());
    }

    #[test]
    fn test_binary_modifier_matching_only() {
        let (rest, m) = binary_modifier(" on(job)").unwrap();
        assert!(rest.is_empty());
        assert!(!m.return_bool);
        assert!(m.matching.is_some());
    }

    #[test]
    fn test_binary_modifier_bool_and_matching() {
        let (rest, m) = binary_modifier(" bool on(job)").unwrap();
        assert!(rest.is_empty());
        assert!(m.return_bool);
        assert!(m.matching.is_some());
    }

    #[test]
    fn test_binary_modifier_fails_on_empty() {
        assert!(binary_modifier("foo").is_err());
    }

    // Display tests
    #[test]
    fn test_vector_matching_display() {
        let vm = VectorMatching {
            op: VectorMatchingOp::On,
            labels: vec!["job".to_string()],
            group: None,
        };
        assert_eq!(vm.to_string(), "on (job)");

        let vm = VectorMatching {
            op: VectorMatchingOp::Ignoring,
            labels: vec!["job".to_string(), "instance".to_string()],
            group: Some(GroupModifier {
                side: GroupSide::Left,
                labels: vec![],
            }),
        };
        // Empty group labels: no parens needed
        assert_eq!(vm.to_string(), "ignoring (job, instance) group_left");
    }

    #[test]
    fn test_binary_modifier_display() {
        let m = BinaryModifier {
            return_bool: true,
            matching: None,
        };
        assert_eq!(m.to_string(), "bool");

        let m = BinaryModifier {
            return_bool: false,
            matching: Some(VectorMatching {
                op: VectorMatchingOp::On,
                labels: vec!["job".to_string()],
                group: None,
            }),
        };
        assert_eq!(m.to_string(), "on (job)");
    }
}
