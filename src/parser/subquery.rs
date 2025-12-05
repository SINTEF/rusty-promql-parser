//! Subquery expression parsing for PromQL
//!
//! Subqueries allow evaluating an instant vector expression over a time range,
//! producing a range vector. This is useful for applying range vector functions
//! (like `avg_over_time`) to the output of other queries.
//!
//! Syntax: `expr[range:step]` or `expr[range:]`
//!
//! - `range` - The lookback window (required)
//! - `step` - The evaluation interval (optional, uses default if omitted)
//!
//! Examples:
//! - `some_metric[5m:1m]` - Evaluate every minute over 5 minutes
//! - `some_metric[5m:]` - Evaluate at default interval over 5 minutes
//! - `rate(http_requests[5m])[30m:1m]` - Rate over 5m, sampled every minute for 30m
//! - `avg_over_time(rate(http_requests[5m])[30m:1m])` - Average of rates

use nom::{
    IResult, Parser,
    character::complete::char,
    combinator::{map, opt, peek, recognize},
    sequence::delimited,
};

use crate::ast::{Expr, SubqueryExpr};
use crate::lexer::duration::{Duration, duration};
use crate::parser::selector::parse_modifiers;

/// Parse a subquery range: `[range:step]` or `[range:]`
///
/// Returns (range, optional_step)
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::parser::subquery::subquery_range;
///
/// let (rest, (range, step)) = subquery_range("[5m:1m]").unwrap();
/// assert!(rest.is_empty());
/// assert_eq!(range.as_millis(), 5 * 60 * 1000);
/// assert_eq!(step.unwrap().as_millis(), 60 * 1000);
///
/// let (rest, (range, step)) = subquery_range("[30m:]").unwrap();
/// assert!(rest.is_empty());
/// assert_eq!(range.as_millis(), 30 * 60 * 1000);
/// assert!(step.is_none());
/// ```
pub fn subquery_range(input: &str) -> IResult<&str, (Duration, Option<Duration>)> {
    delimited(
        char('['),
        map((duration, char(':'), opt(duration)), |(range, _, step)| {
            (range, step)
        }),
        char(']'),
    )
    .parse(input)
}

/// Try to parse a subquery suffix on an expression
///
/// This function attempts to parse `[range:step]` followed by optional modifiers.
/// Returns None if the input doesn't start with a subquery bracket.
///
/// Note: This only parses the subquery part, not the inner expression.
/// The caller is responsible for providing the already-parsed inner expression.
pub fn try_parse_subquery(input: &str, expr: Expr) -> IResult<&str, SubqueryExpr> {
    map(
        (subquery_range, parse_modifiers),
        move |((range, step), (at, offset))| SubqueryExpr {
            expr: expr.clone(),
            range,
            step,
            offset,
            at,
        },
    )
    .parse(input)
}

/// Check if the input looks like a subquery bracket `[duration:`
///
/// This is useful for distinguishing between:
/// - Matrix selector: `metric[5m]`
/// - Subquery: `metric[5m:]` or `metric[5m:1m]`
///
/// The key difference is the presence of `:` after the first duration.
pub fn looks_like_subquery(input: &str) -> bool {
    // Pattern: '[' duration ':'
    peek(recognize((char('['), duration, char(':'))))
        .parse(input)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::selector::VectorSelector;

    #[test]
    fn test_subquery_range_with_step() {
        let (rest, (range, step)) = subquery_range("[5m:1m]").unwrap();
        assert!(rest.is_empty());
        assert_eq!(range.as_millis(), 5 * 60 * 1000);
        assert_eq!(step.unwrap().as_millis(), 60 * 1000);
    }

    #[test]
    fn test_subquery_range_without_step() {
        let (rest, (range, step)) = subquery_range("[30m:]").unwrap();
        assert!(rest.is_empty());
        assert_eq!(range.as_millis(), 30 * 60 * 1000);
        assert!(step.is_none());
    }

    #[test]
    fn test_subquery_range_various_durations() {
        let (_, (range, step)) = subquery_range("[1h:5m]").unwrap();
        assert_eq!(range.as_millis(), 60 * 60 * 1000);
        assert_eq!(step.unwrap().as_millis(), 5 * 60 * 1000);

        let (_, (range, step)) = subquery_range("[1d:1h]").unwrap();
        assert_eq!(range.as_millis(), 24 * 60 * 60 * 1000);
        assert_eq!(step.unwrap().as_millis(), 60 * 60 * 1000);

        let (_, (range, step)) = subquery_range("[1w:]").unwrap();
        assert_eq!(range.as_millis(), 7 * 24 * 60 * 60 * 1000);
        assert!(step.is_none());
    }

    #[test]
    fn test_subquery_range_compound_duration() {
        let (_, (range, step)) = subquery_range("[1h30m:5m30s]").unwrap();
        assert_eq!(range.as_millis(), (60 + 30) * 60 * 1000);
        assert_eq!(step.unwrap().as_millis(), (5 * 60 + 30) * 1000);
    }

    #[test]
    fn test_subquery_range_invalid_no_colon() {
        // This is a matrix selector syntax, not subquery
        assert!(subquery_range("[5m]").is_err());
    }

    #[test]
    fn test_subquery_range_invalid_empty() {
        assert!(subquery_range("[]").is_err());
        assert!(subquery_range("[:]").is_err());
    }

    #[test]
    fn test_try_parse_subquery() {
        let expr = Expr::VectorSelector(VectorSelector::new("metric"));
        let (rest, sq) = try_parse_subquery("[5m:1m]", expr).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sq.range.as_millis(), 5 * 60 * 1000);
        assert_eq!(sq.step.unwrap().as_millis(), 60 * 1000);
    }

    #[test]
    fn test_try_parse_subquery_with_offset() {
        let expr = Expr::VectorSelector(VectorSelector::new("metric"));
        let (rest, sq) = try_parse_subquery("[5m:1m] offset 10m", expr).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sq.offset.unwrap().as_millis(), 10 * 60 * 1000);
    }

    #[test]
    fn test_try_parse_subquery_with_at() {
        let expr = Expr::VectorSelector(VectorSelector::new("metric"));
        let (rest, sq) = try_parse_subquery("[5m:1m] @ 1609459200", expr).unwrap();
        assert!(rest.is_empty());
        assert!(sq.at.is_some());
    }

    #[test]
    fn test_try_parse_subquery_with_both_modifiers() {
        let expr = Expr::VectorSelector(VectorSelector::new("metric"));
        let (rest, sq) = try_parse_subquery("[5m:1m] @ 1609459200 offset 10m", expr).unwrap();
        assert!(rest.is_empty());
        assert!(sq.at.is_some());
        assert!(sq.offset.is_some());

        // Also test the other order
        let expr = Expr::VectorSelector(VectorSelector::new("metric"));
        let (rest, sq) = try_parse_subquery("[5m:1m] offset 10m @ 1609459200", expr).unwrap();
        assert!(rest.is_empty());
        assert!(sq.at.is_some());
        assert!(sq.offset.is_some());
    }

    #[test]
    fn test_looks_like_subquery() {
        // Subquery syntax
        assert!(looks_like_subquery("[5m:]"));
        assert!(looks_like_subquery("[5m:1m]"));
        assert!(looks_like_subquery("[1h30m:5m]"));

        // Matrix selector syntax (not subquery)
        assert!(!looks_like_subquery("[5m]"));
        assert!(!looks_like_subquery("[1h]"));

        // Not brackets at all
        assert!(!looks_like_subquery("foo"));
        assert!(!looks_like_subquery(""));
        assert!(!looks_like_subquery("(5m)"));
    }

    #[test]
    fn test_subquery_expr_display() {
        let sq = SubqueryExpr {
            expr: Expr::VectorSelector(VectorSelector::new("metric")),
            range: Duration::from_secs(300),
            step: Some(Duration::from_secs(60)),
            offset: None,
            at: None,
        };
        assert_eq!(sq.to_string(), "metric[5m:1m]");

        let sq = SubqueryExpr {
            expr: Expr::VectorSelector(VectorSelector::new("metric")),
            range: Duration::from_secs(300),
            step: None,
            offset: None,
            at: None,
        };
        assert_eq!(sq.to_string(), "metric[5m:]");

        let sq = SubqueryExpr {
            expr: Expr::VectorSelector(VectorSelector::new("metric")),
            range: Duration::from_secs(300),
            step: Some(Duration::from_secs(60)),
            offset: Some(Duration::from_secs(600)),
            at: None,
        };
        assert_eq!(sq.to_string(), "metric[5m:1m] offset 10m");
    }
}
