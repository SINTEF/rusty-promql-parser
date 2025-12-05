//! Aggregation grouping clause parsing for PromQL.
//!
//! This module handles parsing the grouping clauses used with aggregation operators:
//!
//! - `by (label1, label2)` - Group by specific labels, dropping all others
//! - `without (label1, label2)` - Drop specific labels, keeping all others
//!
//! # Supported Aggregation Operators
//!
//! These operators support grouping clauses:
//! `sum`, `avg`, `count`, `min`, `max`, `group`, `stddev`, `stdvar`,
//! `topk`, `bottomk`, `count_values`, `quantile`, `limitk`, `limit_ratio`
//!
//! # Examples
//!
//! ```rust
//! use rusty_promql_parser::parser::aggregation::{grouping, GroupingAction};
//!
//! let (rest, g) = grouping("by (job, instance)").unwrap();
//! assert!(rest.is_empty());
//! assert_eq!(g.action, GroupingAction::By);
//! assert_eq!(g.labels, vec!["job", "instance"]);
//!
//! let (rest, g) = grouping("without (instance)").unwrap();
//! assert!(rest.is_empty());
//! assert_eq!(g.action, GroupingAction::Without);
//! ```

use std::fmt;

use nom::{
    IResult, Parser, branch::alt, bytes::complete::tag_no_case, character::complete::char,
    multi::separated_list0, sequence::delimited,
};

use crate::lexer::{identifier::label_name, whitespace::ws_opt};

/// The action for aggregation grouping: `by` or `without`.
///
/// - [`GroupingAction::By`]: Group results by the specified labels only
/// - [`GroupingAction::Without`]: Group results by all labels except those specified
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupingAction {
    /// Group by specific labels, dropping all others.
    ///
    /// Example: `sum by (job) (http_requests)` groups by `job` label only.
    By,
    /// Drop specific labels, keeping all others.
    ///
    /// Example: `sum without (instance) (http_requests)` keeps all labels except `instance`.
    Without,
}

impl fmt::Display for GroupingAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupingAction::By => write!(f, "by"),
            GroupingAction::Without => write!(f, "without"),
        }
    }
}

/// Grouping clause for aggregation expressions.
///
/// Specifies how to group results when aggregating across time series.
///
/// # Example
///
/// ```rust
/// use rusty_promql_parser::parser::aggregation::{Grouping, GroupingAction};
///
/// let g = Grouping {
///     action: GroupingAction::By,
///     labels: vec!["job".to_string(), "instance".to_string()],
/// };
/// assert_eq!(g.to_string(), "by (job, instance)");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grouping {
    /// The grouping action (by or without).
    pub action: GroupingAction,
    /// The label names to group by/without.
    pub labels: Vec<String>,
}

impl fmt::Display for Grouping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (", self.action)?;
        for (i, label) in self.labels.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", label)?;
        }
        write!(f, ")")
    }
}

/// Parse a grouping clause: `by (label1, label2)` or `without (label1, label2)`
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::parser::aggregation::{grouping, GroupingAction};
///
/// let (rest, g) = grouping("by (job, instance)").unwrap();
/// assert!(rest.is_empty());
/// assert_eq!(g.action, GroupingAction::By);
/// assert_eq!(g.labels, vec!["job", "instance"]);
///
/// let (rest, g) = grouping("without (job)").unwrap();
/// assert!(rest.is_empty());
/// assert_eq!(g.action, GroupingAction::Without);
/// ```
pub fn grouping(input: &str) -> IResult<&str, Grouping> {
    (
        // Parse the action (by or without)
        alt((
            tag_no_case("by").map(|_| GroupingAction::By),
            tag_no_case("without").map(|_| GroupingAction::Without),
        )),
        // Parse: ws "(" ws labels ws ")"
        delimited(
            (ws_opt, char('('), ws_opt),
            separated_list0((ws_opt, char(','), ws_opt), label_name.map(String::from)),
            (ws_opt, char(')')),
        ),
    )
        .map(|(action, labels)| Grouping { action, labels })
        .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grouping_by() {
        let (rest, g) = grouping("by (job)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(g.action, GroupingAction::By);
        assert_eq!(g.labels, vec!["job"]);
    }

    #[test]
    fn test_grouping_without() {
        let (rest, g) = grouping("without (instance)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(g.action, GroupingAction::Without);
        assert_eq!(g.labels, vec!["instance"]);
    }

    #[test]
    fn test_grouping_multiple_labels() {
        let (rest, g) = grouping("by (job, instance, method)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(g.labels, vec!["job", "instance", "method"]);
    }

    #[test]
    fn test_grouping_empty() {
        let (rest, g) = grouping("by ()").unwrap();
        assert!(rest.is_empty());
        assert!(g.labels.is_empty());
    }

    #[test]
    fn test_grouping_case_insensitive() {
        let (rest, g) = grouping("BY (job)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(g.action, GroupingAction::By);

        let (rest, g) = grouping("WITHOUT (job)").unwrap();
        assert!(rest.is_empty());
        assert_eq!(g.action, GroupingAction::Without);
    }

    #[test]
    fn test_grouping_display() {
        let g = Grouping {
            action: GroupingAction::By,
            labels: vec!["job".to_string(), "instance".to_string()],
        };
        assert_eq!(format!("{}", g), "by (job, instance)");

        let g = Grouping {
            action: GroupingAction::Without,
            labels: vec!["job".to_string()],
        };
        assert_eq!(format!("{}", g), "without (job)");
    }
}
