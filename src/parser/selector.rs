//! Vector selector parsing for PromQL
//!
//! A vector selector selects a set of time series and a single sample value
//! for each at a given timestamp (instant).
//!
//! Syntax:
//! ```text
//! metric_name
//! metric_name{label_matchers}
//! {label_matchers}
//! ```
//!
//! Label matchers:
//! - `=`  : equality
//! - `!=` : inequality
//! - `=~` : regex match
//! - `!~` : regex not match

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    multi::{fold_many0, separated_list0},
    sequence::{delimited, terminated},
};

use crate::lexer::{
    duration::{Duration, duration, signed_duration},
    identifier::{label_name, metric_name},
    number::number,
    string::string_literal,
    whitespace::ws_opt,
};

/// @ modifier for timestamp pinning
///
/// The @ modifier allows pinning a query to a specific timestamp,
/// or to the start/end of the evaluation range.
#[derive(Debug, Clone, PartialEq)]
pub enum AtModifier {
    /// Pin to a specific Unix timestamp (in milliseconds)
    Timestamp(i64),
    /// Pin to the start of the evaluation range: `@ start()`
    Start,
    /// Pin to the end of the evaluation range: `@ end()`
    End,
}

impl std::fmt::Display for AtModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtModifier::Timestamp(ts) => {
                // Convert milliseconds to seconds with 3 decimal places
                let secs = *ts as f64 / 1000.0;
                write!(f, "@ {:.3}", secs)
            }
            AtModifier::Start => write!(f, "@ start()"),
            AtModifier::End => write!(f, "@ end()"),
        }
    }
}

/// Label matching operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelMatchOp {
    /// `=` - Exact string equality
    Equal,
    /// `!=` - String inequality
    NotEqual,
    /// `=~` - Regex match
    RegexMatch,
    /// `!~` - Regex not match
    RegexNotMatch,
}

impl LabelMatchOp {
    /// Get the operator as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            LabelMatchOp::Equal => "=",
            LabelMatchOp::NotEqual => "!=",
            LabelMatchOp::RegexMatch => "=~",
            LabelMatchOp::RegexNotMatch => "!~",
        }
    }

    /// Check if this is a negative matcher (!=, !~)
    pub fn is_negative(&self) -> bool {
        matches!(self, LabelMatchOp::NotEqual | LabelMatchOp::RegexNotMatch)
    }

    /// Check if this is a regex matcher (=~, !~)
    pub fn is_regex(&self) -> bool {
        matches!(self, LabelMatchOp::RegexMatch | LabelMatchOp::RegexNotMatch)
    }
}

impl std::fmt::Display for LabelMatchOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single label matcher
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelMatcher {
    /// Label name (e.g., "job", "__name__")
    pub name: String,
    /// Matching operator
    pub op: LabelMatchOp,
    /// Value to match against
    pub value: String,
}

impl LabelMatcher {
    /// Create a new label matcher
    pub fn new(name: impl Into<String>, op: LabelMatchOp, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            op,
            value: value.into(),
        }
    }

    /// Check if this matcher matches the empty string
    pub fn matches_empty(&self) -> bool {
        match self.op {
            LabelMatchOp::Equal => self.value.is_empty(),
            LabelMatchOp::NotEqual => !self.value.is_empty(),
            LabelMatchOp::RegexMatch => {
                // A regex matches empty if it can match ""
                // Common patterns: "", ".*", "^$", etc.
                self.value.is_empty()
                    || self.value == ".*"
                    || self.value == "^$"
                    || self.value == "^.*$"
            }
            LabelMatchOp::RegexNotMatch => {
                // !~ matches empty if the regex doesn't match ""
                // ".+" requires at least one character, so it doesn't match ""
                self.value == ".+"
            }
        }
    }
}

impl std::fmt::Display for LabelMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}\"{}\"",
            self.name,
            self.op,
            self.value.escape_default()
        )
    }
}

/// A vector selector expression
#[derive(Debug, Clone, PartialEq)]
pub struct VectorSelector {
    /// Metric name (optional if label matchers include __name__)
    pub name: Option<String>,
    /// Label matchers
    pub matchers: Vec<LabelMatcher>,
    /// Offset modifier (e.g., `offset 5m`, `offset -1h`)
    pub offset: Option<Duration>,
    /// @ modifier for timestamp pinning
    pub at: Option<AtModifier>,
}

impl VectorSelector {
    /// Create a new vector selector with just a metric name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            matchers: Vec::new(),
            offset: None,
            at: None,
        }
    }

    /// Create a new vector selector with only label matchers
    pub fn with_matchers(matchers: Vec<LabelMatcher>) -> Self {
        Self {
            name: None,
            matchers,
            offset: None,
            at: None,
        }
    }

    /// Add a label matcher
    pub fn add_matcher(&mut self, matcher: LabelMatcher) {
        self.matchers.push(matcher);
    }

    /// Get all matchers including the implicit __name__ matcher
    pub fn all_matchers(&self) -> Vec<LabelMatcher> {
        let mut result = self.matchers.clone();
        if let Some(ref name) = self.name {
            result.push(LabelMatcher::new(
                "__name__",
                LabelMatchOp::Equal,
                name.clone(),
            ));
        }
        result
    }

    /// Check if this selector has at least one non-empty matcher
    /// (Required for valid selectors to avoid selecting all series)
    pub fn has_non_empty_matcher(&self) -> bool {
        // If we have an explicit metric name, that's a non-empty matcher
        if self.name.is_some() {
            return true;
        }

        // Check if any label matcher doesn't match empty
        self.matchers.iter().any(|m| !m.matches_empty())
    }
}

impl std::fmt::Display for VectorSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref name) = self.name {
            write!(f, "{}", name)?;
        }
        if !self.matchers.is_empty() {
            write!(f, "{{")?;
            for (i, m) in self.matchers.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", m)?;
            }
            write!(f, "}}")?;
        }
        // @ modifier comes before offset in PromQL
        if let Some(ref at) = self.at {
            write!(f, " {}", at)?;
        }
        if let Some(ref offset) = self.offset {
            write!(f, " offset {}", offset)?;
        }
        Ok(())
    }
}

/// A matrix selector expression (range vector)
///
/// A matrix selector selects a range of samples over time for each matching time series.
/// It extends a vector selector with a range duration in square brackets.
///
/// Syntax:
/// ```text
/// metric_name[5m]
/// metric_name{label="value"}[1h]
/// {label="value"}[30s]
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MatrixSelector {
    /// The underlying vector selector
    pub selector: VectorSelector,
    /// The range duration (e.g., 5m, 1h, 30s)
    pub range: Duration,
}

impl MatrixSelector {
    /// Create a new matrix selector from a vector selector and range
    pub fn new(selector: VectorSelector, range: Duration) -> Self {
        Self { selector, range }
    }

    /// Create a matrix selector with just a metric name and range
    pub fn with_name(name: impl Into<String>, range: Duration) -> Self {
        Self {
            selector: VectorSelector::new(name),
            range,
        }
    }

    /// Get the metric name (if any)
    pub fn name(&self) -> Option<&str> {
        self.selector.name.as_deref()
    }

    /// Get the label matchers
    pub fn matchers(&self) -> &[LabelMatcher] {
        &self.selector.matchers
    }

    /// Get the range duration in milliseconds
    pub fn range_millis(&self) -> i64 {
        self.range.as_millis()
    }

    /// Get the offset duration (if any)
    pub fn offset(&self) -> Option<&Duration> {
        self.selector.offset.as_ref()
    }

    /// Get the offset duration in milliseconds (if any)
    pub fn offset_millis(&self) -> Option<i64> {
        self.selector.offset.map(|d| d.as_millis())
    }

    /// Get the @ modifier (if any)
    pub fn at(&self) -> Option<&AtModifier> {
        self.selector.at.as_ref()
    }
}

impl std::fmt::Display for MatrixSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Write name and matchers without offset/at
        if let Some(ref name) = self.selector.name {
            write!(f, "{}", name)?;
        }
        if !self.selector.matchers.is_empty() {
            write!(f, "{{")?;
            for (i, m) in self.selector.matchers.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", m)?;
            }
            write!(f, "}}")?;
        }
        // Write range
        write!(f, "[{}]", self.range)?;
        // Write @ modifier (if any) - comes before offset
        if let Some(ref at) = self.selector.at {
            write!(f, " {}", at)?;
        }
        // Write offset (if any)
        if let Some(ref offset) = self.selector.offset {
            write!(f, " offset {}", offset)?;
        }
        Ok(())
    }
}

/// Parse a range duration in square brackets: `[5m]`, `[1h30m]`
fn range_duration(input: &str) -> IResult<&str, Duration> {
    delimited(char('['), duration, char(']')).parse(input)
}

/// Parse the offset modifier keyword (case-insensitive)
fn offset_keyword(input: &str) -> IResult<&str, &str> {
    alt((tag("offset"), tag("OFFSET"), tag("Offset"))).parse(input)
}

/// Parse an offset modifier: `offset 5m`, `offset -1h`
///
/// The offset modifier shifts the time range of a vector selector back in time.
/// Negative offsets look forward in time (relative to query evaluation time).
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::parser::selector::offset_modifier;
///
/// let (rest, dur) = offset_modifier(" offset 5m").unwrap();
/// assert!(rest.is_empty());
/// assert_eq!(dur.as_millis(), 300_000);
///
/// let (rest, dur) = offset_modifier(" offset -7m").unwrap();
/// assert!(rest.is_empty());
/// assert_eq!(dur.as_millis(), -420_000);
///
/// let (rest, dur) = offset_modifier(" OFFSET 1h30m").unwrap();
/// assert!(rest.is_empty());
/// assert_eq!(dur.as_millis(), 5_400_000);
/// ```
pub fn offset_modifier(input: &str) -> IResult<&str, Duration> {
    let (rest, _) = ws_opt(input)?;
    let (rest, _) = offset_keyword(rest)?;
    let (rest, _) = ws_opt(rest)?;
    signed_duration(rest)
}

/// Parse the @ modifier: `@ <timestamp>`, `@ start()`, `@ end()`
///
/// The @ modifier allows pinning a query to a specific timestamp,
/// or to the start/end of the evaluation range.
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::parser::selector::at_modifier;
///
/// // Timestamp in seconds
/// let (rest, at) = at_modifier(" @ 1603774568").unwrap();
/// assert!(rest.is_empty());
///
/// // start() preprocessor
/// let (rest, at) = at_modifier(" @ start()").unwrap();
/// assert!(rest.is_empty());
///
/// // end() preprocessor
/// let (rest, at) = at_modifier(" @ end()").unwrap();
/// assert!(rest.is_empty());
/// ```
pub fn at_modifier(input: &str) -> IResult<&str, AtModifier> {
    let (rest, _) = ws_opt(input)?;
    let (rest, _) = char('@')(rest)?;
    let (rest, _) = ws_opt(rest)?;

    // Try start() or end() first
    if let Ok((rest, _)) = tag::<&str, &str, nom::error::Error<&str>>("start()")(rest) {
        return Ok((rest, AtModifier::Start));
    }
    if let Ok((rest, _)) = tag::<&str, &str, nom::error::Error<&str>>("end()")(rest) {
        return Ok((rest, AtModifier::End));
    }

    // Otherwise parse a number (timestamp in seconds)
    let (rest, ts) = number(rest)?;

    // Check for invalid timestamps (Inf, NaN)
    if ts.is_infinite() || ts.is_nan() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }

    // Convert seconds to milliseconds, rounding to nearest
    let ts_ms = (ts * 1000.0).round() as i64;
    Ok((rest, AtModifier::Timestamp(ts_ms)))
}

/// Parse a matrix selector (range vector)
///
/// A matrix selector consists of a vector selector followed by a range duration
/// in square brackets.
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::parser::selector::matrix_selector;
///
/// // Simple metric with range
/// let (rest, sel) = matrix_selector("http_requests_total[5m]").unwrap();
/// assert!(rest.is_empty());
/// assert_eq!(sel.name(), Some("http_requests_total"));
/// assert_eq!(sel.range_millis(), 5 * 60 * 1000);
///
/// // With label matchers
/// let (rest, sel) = matrix_selector(r#"http_requests_total{job="api"}[1h]"#).unwrap();
/// assert!(rest.is_empty());
/// assert_eq!(sel.matchers().len(), 1);
/// ```
pub fn matrix_selector(input: &str) -> IResult<&str, MatrixSelector> {
    let (rest, mut selector) = base_vector_selector(input)?;
    let (rest, range) = range_duration(rest)?;
    // Parse @ and offset modifiers (in any order)
    let (rest, (at, offset)) = parse_modifiers(rest)?;
    selector.at = at;
    selector.offset = offset;

    Ok((rest, MatrixSelector::new(selector, range)))
}

/// Modifier type for fold_many0
enum Modifier {
    At(AtModifier),
    Offset(Duration),
}

/// Parse @ and offset modifiers in any order
/// Returns (at_modifier, offset_modifier)
pub fn parse_modifiers(input: &str) -> IResult<&str, (Option<AtModifier>, Option<Duration>)> {
    fold_many0(
        alt((
            at_modifier.map(Modifier::At),
            offset_modifier.map(Modifier::Offset),
        )),
        || (None, None),
        |(at, offset), modifier| match modifier {
            Modifier::At(a) if at.is_none() => (Some(a), offset),
            Modifier::Offset(o) if offset.is_none() => (at, Some(o)),
            // Ignore duplicates
            _ => (at, offset),
        },
    )
    .parse(input)
}

/// Parse a label match operator
fn label_match_op(input: &str) -> IResult<&str, LabelMatchOp> {
    alt((
        map(tag("!="), |_| LabelMatchOp::NotEqual),
        map(tag("!~"), |_| LabelMatchOp::RegexNotMatch),
        map(tag("=~"), |_| LabelMatchOp::RegexMatch),
        map(tag("="), |_| LabelMatchOp::Equal),
    ))
    .parse(input)
}

/// Parse a single label matcher: `label_name op "value"`
fn label_matcher(input: &str) -> IResult<&str, LabelMatcher> {
    let (input, _) = ws_opt(input)?;
    let (input, name) = label_name(input)?;
    let (input, _) = ws_opt(input)?;
    let (input, op) = label_match_op(input)?;
    let (input, _) = ws_opt(input)?;
    let (input, value) = string_literal(input)?;

    Ok((input, LabelMatcher::new(name.to_string(), op, value)))
}

/// Parse a quoted metric name as a matcher: `"metric_name"` inside braces
fn quoted_metric_matcher(input: &str) -> IResult<&str, LabelMatcher> {
    let (input, _) = ws_opt(input)?;
    let (input, name) = string_literal(input)?;

    Ok((
        input,
        LabelMatcher::new("__name__", LabelMatchOp::Equal, name),
    ))
}

/// Parse a matcher item (either a label matcher or quoted metric name)
fn matcher_item(input: &str) -> IResult<&str, LabelMatcher> {
    alt((label_matcher, quoted_metric_matcher)).parse(input)
}

/// Parse label matchers inside braces: `{label="value", ...}`
pub fn label_matchers(input: &str) -> IResult<&str, Vec<LabelMatcher>> {
    delimited(
        (char('{'), ws_opt),
        terminated(
            separated_list0(delimited(ws_opt, char(','), ws_opt), matcher_item),
            opt((ws_opt, char(','))), // Allow trailing comma
        ),
        (ws_opt, char('}')),
    )
    .parse(input)
}

/// Parse a vector selector
///
/// Supports:
/// - `metric_name` - Simple metric name
/// - `metric_name{label="value"}` - Metric with label matchers
/// - `{label="value"}` - Label matchers only
/// - `{"metric_name"}` - Quoted metric name in braces
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::parser::selector::vector_selector;
///
/// let (_, sel) = vector_selector("http_requests_total").unwrap();
/// assert_eq!(sel.name, Some("http_requests_total".to_string()));
///
/// let (_, sel) = vector_selector(r#"foo{bar="baz"}"#).unwrap();
/// assert_eq!(sel.name, Some("foo".to_string()));
/// assert_eq!(sel.matchers.len(), 1);
/// ```
pub fn vector_selector(input: &str) -> IResult<&str, VectorSelector> {
    let (rest, mut selector) = base_vector_selector(input)?;
    // Parse @ and offset modifiers (in any order)
    let (rest, (at, offset)) = parse_modifiers(rest)?;
    selector.at = at;
    selector.offset = offset;
    Ok((rest, selector))
}

/// Parse a vector selector without offset modifier.
/// This is used internally by matrix_selector which handles offset after the range.
pub fn base_vector_selector(input: &str) -> IResult<&str, VectorSelector> {
    // Try to parse metric name first
    let name_result = metric_name(input);

    match name_result {
        Ok((rest, name)) => {
            // Check for label matchers
            let (rest, matchers) = opt(label_matchers).parse(rest)?;
            Ok((
                rest,
                VectorSelector {
                    name: Some(name.to_string()),
                    matchers: matchers.unwrap_or_default(),
                    offset: None,
                    at: None,
                },
            ))
        }
        Err(_) => {
            // No metric name, try label matchers only
            let (rest, matchers) = label_matchers(input)?;

            // Check if any matcher is a __name__ matcher (quoted metric name)
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

            Ok((
                rest,
                VectorSelector {
                    name,
                    matchers: other_matchers,
                    offset: None,
                    at: None,
                },
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // LabelMatchOp tests
    #[test]
    fn test_label_match_op_parse() {
        assert_eq!(label_match_op("=").unwrap().1, LabelMatchOp::Equal);
        assert_eq!(label_match_op("!=").unwrap().1, LabelMatchOp::NotEqual);
        assert_eq!(label_match_op("=~").unwrap().1, LabelMatchOp::RegexMatch);
        assert_eq!(label_match_op("!~").unwrap().1, LabelMatchOp::RegexNotMatch);
    }

    #[test]
    fn test_label_match_op_display() {
        assert_eq!(LabelMatchOp::Equal.to_string(), "=");
        assert_eq!(LabelMatchOp::NotEqual.to_string(), "!=");
        assert_eq!(LabelMatchOp::RegexMatch.to_string(), "=~");
        assert_eq!(LabelMatchOp::RegexNotMatch.to_string(), "!~");
    }

    #[test]
    fn test_label_match_op_properties() {
        assert!(!LabelMatchOp::Equal.is_negative());
        assert!(LabelMatchOp::NotEqual.is_negative());
        assert!(!LabelMatchOp::RegexMatch.is_negative());
        assert!(LabelMatchOp::RegexNotMatch.is_negative());

        assert!(!LabelMatchOp::Equal.is_regex());
        assert!(!LabelMatchOp::NotEqual.is_regex());
        assert!(LabelMatchOp::RegexMatch.is_regex());
        assert!(LabelMatchOp::RegexNotMatch.is_regex());
    }

    // LabelMatcher tests
    #[test]
    fn test_label_matcher_parse() {
        let (rest, m) = label_matcher(r#"job="prometheus""#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(m.name, "job");
        assert_eq!(m.op, LabelMatchOp::Equal);
        assert_eq!(m.value, "prometheus");
    }

    #[test]
    fn test_label_matcher_parse_with_spaces() {
        let (rest, m) = label_matcher(r#"  job  =  "prometheus"  "#).unwrap();
        assert_eq!(rest, "  "); // Trailing space not consumed
        assert_eq!(m.name, "job");
        assert_eq!(m.value, "prometheus");
    }

    #[test]
    fn test_label_matcher_not_equal() {
        let (_, m) = label_matcher(r#"env!="prod""#).unwrap();
        assert_eq!(m.op, LabelMatchOp::NotEqual);
    }

    #[test]
    fn test_label_matcher_regex() {
        let (_, m) = label_matcher(r#"path=~"/api/.*""#).unwrap();
        assert_eq!(m.op, LabelMatchOp::RegexMatch);
        assert_eq!(m.value, "/api/.*");
    }

    #[test]
    fn test_label_matcher_regex_not() {
        let (_, m) = label_matcher(r#"status!~"5..""#).unwrap();
        assert_eq!(m.op, LabelMatchOp::RegexNotMatch);
    }

    #[test]
    fn test_label_matcher_matches_empty() {
        // Equal empty matches empty
        assert!(LabelMatcher::new("a", LabelMatchOp::Equal, "").matches_empty());
        // Equal non-empty doesn't match empty
        assert!(!LabelMatcher::new("a", LabelMatchOp::Equal, "foo").matches_empty());
        // NotEqual empty doesn't match empty
        assert!(!LabelMatcher::new("a", LabelMatchOp::NotEqual, "").matches_empty());
        // NotEqual non-empty matches empty
        assert!(LabelMatcher::new("a", LabelMatchOp::NotEqual, "foo").matches_empty());
        // Regex .* matches empty
        assert!(LabelMatcher::new("a", LabelMatchOp::RegexMatch, ".*").matches_empty());
        // Regex .+ doesn't match empty
        assert!(!LabelMatcher::new("a", LabelMatchOp::RegexMatch, ".+").matches_empty());
        // Not regex .+ matches empty
        assert!(LabelMatcher::new("a", LabelMatchOp::RegexNotMatch, ".+").matches_empty());
    }

    // VectorSelector tests
    #[test]
    fn test_vector_selector_simple_name() {
        let (rest, sel) = vector_selector("foo").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert!(sel.matchers.is_empty());
    }

    #[test]
    fn test_vector_selector_with_underscore() {
        let (rest, sel) = vector_selector("http_requests_total").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("http_requests_total".to_string()));
    }

    #[test]
    fn test_vector_selector_with_colon() {
        let (rest, sel) = vector_selector("foo:bar:baz").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo:bar:baz".to_string()));
    }

    #[test]
    fn test_vector_selector_with_label() {
        let (rest, sel) = vector_selector(r#"foo{bar="baz"}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert_eq!(sel.matchers.len(), 1);
        assert_eq!(sel.matchers[0].name, "bar");
        assert_eq!(sel.matchers[0].value, "baz");
    }

    #[test]
    fn test_vector_selector_multiple_labels() {
        let (rest, sel) = vector_selector(r#"foo{a="b", c="d"}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert_eq!(sel.matchers.len(), 2);
        assert_eq!(sel.matchers[0].name, "a");
        assert_eq!(sel.matchers[1].name, "c");
    }

    #[test]
    fn test_vector_selector_trailing_comma() {
        let (rest, sel) = vector_selector(r#"foo{a="b",}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.matchers.len(), 1);
    }

    #[test]
    fn test_vector_selector_labels_only() {
        let (rest, sel) = vector_selector(r#"{job="prometheus"}"#).unwrap();
        assert!(rest.is_empty());
        assert!(sel.name.is_none());
        assert_eq!(sel.matchers.len(), 1);
        assert_eq!(sel.matchers[0].name, "job");
    }

    #[test]
    fn test_vector_selector_quoted_metric_name() {
        let (rest, sel) = vector_selector(r#"{"foo"}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert!(sel.matchers.is_empty());
    }

    #[test]
    fn test_vector_selector_quoted_metric_with_labels() {
        let (rest, sel) = vector_selector(r#"{"foo", bar="baz"}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert_eq!(sel.matchers.len(), 1);
    }

    #[test]
    fn test_vector_selector_all_operators() {
        let (rest, sel) = vector_selector(r#"foo{a="b", c!="d", e=~"f", g!~"h"}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.matchers.len(), 4);
        assert_eq!(sel.matchers[0].op, LabelMatchOp::Equal);
        assert_eq!(sel.matchers[1].op, LabelMatchOp::NotEqual);
        assert_eq!(sel.matchers[2].op, LabelMatchOp::RegexMatch);
        assert_eq!(sel.matchers[3].op, LabelMatchOp::RegexNotMatch);
    }

    #[test]
    fn test_vector_selector_has_non_empty_matcher() {
        // With metric name - always has non-empty
        let sel = VectorSelector::new("foo");
        assert!(sel.has_non_empty_matcher());

        // With non-empty label value
        let mut sel = VectorSelector::with_matchers(vec![]);
        sel.add_matcher(LabelMatcher::new("job", LabelMatchOp::Equal, "test"));
        assert!(sel.has_non_empty_matcher());

        // With only empty matcher
        let sel =
            VectorSelector::with_matchers(vec![LabelMatcher::new("x", LabelMatchOp::Equal, "")]);
        assert!(!sel.has_non_empty_matcher());
    }

    #[test]
    fn test_vector_selector_display() {
        let mut sel = VectorSelector::new("foo");
        assert_eq!(sel.to_string(), "foo");

        sel.add_matcher(LabelMatcher::new("bar", LabelMatchOp::Equal, "baz"));
        assert_eq!(sel.to_string(), r#"foo{bar="baz"}"#);
    }

    #[test]
    fn test_vector_selector_single_quoted() {
        let (rest, sel) = vector_selector(r#"foo{bar='baz'}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.matchers[0].value, "baz");
    }

    #[test]
    fn test_vector_selector_backtick() {
        let (rest, sel) = vector_selector(r#"foo{bar=`baz`}"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.matchers[0].value, "baz");
    }

    #[test]
    fn test_vector_selector_keyword_as_metric() {
        // Keywords can be used as metric names
        for keyword in [
            "sum", "min", "max", "avg", "count", "offset", "by", "without",
        ] {
            let result = vector_selector(keyword);
            assert!(
                result.is_ok(),
                "Failed to parse keyword as metric: {}",
                keyword
            );
            let (_, sel) = result.unwrap();
            assert_eq!(sel.name, Some(keyword.to_string()));
        }
    }

    #[test]
    fn test_vector_selector_empty_braces() {
        // Empty braces should parse but result in no matchers
        let (rest, sel) = vector_selector("{}").unwrap();
        assert!(rest.is_empty());
        assert!(sel.name.is_none());
        assert!(sel.matchers.is_empty());
        // Note: validation that this is invalid should happen at a higher level
    }

    // MatrixSelector tests
    #[test]
    fn test_matrix_selector_simple() {
        let (rest, sel) = matrix_selector("foo[5m]").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name(), Some("foo"));
        assert_eq!(sel.range_millis(), 5 * 60 * 1000);
    }

    #[test]
    fn test_matrix_selector_with_labels() {
        let (rest, sel) = matrix_selector(r#"foo{bar="baz"}[5m]"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name(), Some("foo"));
        assert_eq!(sel.matchers().len(), 1);
        assert_eq!(sel.range_millis(), 5 * 60 * 1000);
    }

    #[test]
    fn test_matrix_selector_various_durations() {
        // Seconds
        let (_, sel) = matrix_selector("foo[30s]").unwrap();
        assert_eq!(sel.range_millis(), 30 * 1000);

        // Minutes
        let (_, sel) = matrix_selector("foo[5m]").unwrap();
        assert_eq!(sel.range_millis(), 5 * 60 * 1000);

        // Hours
        let (_, sel) = matrix_selector("foo[1h]").unwrap();
        assert_eq!(sel.range_millis(), 60 * 60 * 1000);

        // Days
        let (_, sel) = matrix_selector("foo[1d]").unwrap();
        assert_eq!(sel.range_millis(), 24 * 60 * 60 * 1000);

        // Weeks
        let (_, sel) = matrix_selector("foo[1w]").unwrap();
        assert_eq!(sel.range_millis(), 7 * 24 * 60 * 60 * 1000);

        // Milliseconds
        let (_, sel) = matrix_selector("foo[100ms]").unwrap();
        assert_eq!(sel.range_millis(), 100);
    }

    #[test]
    fn test_matrix_selector_compound_duration() {
        // 1h30m = 90 minutes
        let (rest, sel) = matrix_selector("foo[1h30m]").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.range_millis(), (60 + 30) * 60 * 1000);
    }

    #[test]
    fn test_matrix_selector_labels_only() {
        let (rest, sel) = matrix_selector(r#"{job="prometheus"}[5m]"#).unwrap();
        assert!(rest.is_empty());
        assert!(sel.name().is_none());
        assert_eq!(sel.matchers().len(), 1);
    }

    #[test]
    fn test_matrix_selector_display() {
        let sel = MatrixSelector::with_name("foo", Duration::from_secs(300));
        assert_eq!(sel.to_string(), "foo[5m]");
    }

    #[test]
    fn test_matrix_selector_display_with_labels() {
        let mut vs = VectorSelector::new("foo");
        vs.add_matcher(LabelMatcher::new("bar", LabelMatchOp::Equal, "baz"));
        let sel = MatrixSelector::new(vs, Duration::from_secs(300));
        assert_eq!(sel.to_string(), r#"foo{bar="baz"}[5m]"#);
    }

    #[test]
    fn test_matrix_selector_no_range_fails() {
        // Vector selector without range should fail for matrix_selector
        let result = matrix_selector("foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_matrix_selector_empty_range_fails() {
        let result = matrix_selector("foo[]");
        assert!(result.is_err());
    }

    // Offset modifier tests
    #[test]
    fn test_offset_modifier_basic() {
        let (rest, dur) = offset_modifier(" offset 5m").unwrap();
        assert!(rest.is_empty());
        assert_eq!(dur.as_millis(), 5 * 60 * 1000);
    }

    #[test]
    fn test_offset_modifier_negative() {
        let (rest, dur) = offset_modifier(" offset -7m").unwrap();
        assert!(rest.is_empty());
        assert_eq!(dur.as_millis(), -7 * 60 * 1000);
    }

    #[test]
    fn test_offset_modifier_uppercase() {
        let (rest, dur) = offset_modifier(" OFFSET 1h30m").unwrap();
        assert!(rest.is_empty());
        assert_eq!(dur.as_millis(), 90 * 60 * 1000);
    }

    #[test]
    fn test_offset_modifier_complex_duration() {
        let (rest, dur) = offset_modifier(" OFFSET 1m30ms").unwrap();
        assert!(rest.is_empty());
        assert_eq!(dur.as_millis(), 60 * 1000 + 30);
    }

    #[test]
    fn test_vector_selector_with_offset() {
        let (rest, sel) = vector_selector("foo offset 5m").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert_eq!(sel.offset.unwrap().as_millis(), 5 * 60 * 1000);
    }

    #[test]
    fn test_vector_selector_with_negative_offset() {
        let (rest, sel) = vector_selector("foo offset -7m").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert_eq!(sel.offset.unwrap().as_millis(), -7 * 60 * 1000);
    }

    #[test]
    fn test_vector_selector_with_labels_and_offset() {
        let (rest, sel) = vector_selector(r#"foo{bar="baz"} offset 1h"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert_eq!(sel.matchers.len(), 1);
        assert_eq!(sel.offset.unwrap().as_millis(), 60 * 60 * 1000);
    }

    #[test]
    fn test_vector_selector_display_with_offset() {
        let mut sel = VectorSelector::new("foo");
        sel.offset = Some(Duration::from_secs(300));
        assert_eq!(sel.to_string(), "foo offset 5m");
    }

    #[test]
    fn test_matrix_selector_with_offset() {
        let (rest, sel) = matrix_selector("foo[5m] offset 1h").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name(), Some("foo"));
        assert_eq!(sel.range_millis(), 5 * 60 * 1000);
        assert_eq!(sel.offset_millis(), Some(60 * 60 * 1000));
    }

    #[test]
    fn test_matrix_selector_with_labels_and_offset() {
        let (rest, sel) = matrix_selector(r#"foo{bar="baz"}[5m] offset 30m"#).unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name(), Some("foo"));
        assert_eq!(sel.matchers().len(), 1);
        assert_eq!(sel.range_millis(), 5 * 60 * 1000);
        assert_eq!(sel.offset_millis(), Some(30 * 60 * 1000));
    }

    #[test]
    fn test_matrix_selector_with_negative_offset() {
        let (rest, sel) = matrix_selector("foo[5m] offset -1h").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.offset_millis(), Some(-60 * 60 * 1000));
    }

    #[test]
    fn test_matrix_selector_display_with_offset() {
        let mut vs = VectorSelector::new("foo");
        vs.offset = Some(Duration::from_secs(3600));
        let sel = MatrixSelector::new(vs, Duration::from_secs(300));
        assert_eq!(sel.to_string(), "foo[5m] offset 1h");
    }

    // @ modifier tests
    #[test]
    fn test_at_modifier_timestamp() {
        let (rest, at) = at_modifier(" @ 1603774568").unwrap();
        assert!(rest.is_empty());
        assert_eq!(at, AtModifier::Timestamp(1_603_774_568_000));
    }

    #[test]
    fn test_at_modifier_negative_timestamp() {
        let (rest, at) = at_modifier(" @ -100").unwrap();
        assert!(rest.is_empty());
        assert_eq!(at, AtModifier::Timestamp(-100_000));
    }

    #[test]
    fn test_at_modifier_float_timestamp() {
        let (rest, at) = at_modifier(" @ 3.33").unwrap();
        assert!(rest.is_empty());
        assert_eq!(at, AtModifier::Timestamp(3_330));
    }

    #[test]
    fn test_at_modifier_start() {
        let (rest, at) = at_modifier(" @ start()").unwrap();
        assert!(rest.is_empty());
        assert_eq!(at, AtModifier::Start);
    }

    #[test]
    fn test_at_modifier_end() {
        let (rest, at) = at_modifier(" @ end()").unwrap();
        assert!(rest.is_empty());
        assert_eq!(at, AtModifier::End);
    }

    #[test]
    fn test_at_modifier_display_timestamp() {
        let at = AtModifier::Timestamp(1_603_774_568_000);
        assert_eq!(at.to_string(), "@ 1603774568.000");
    }

    #[test]
    fn test_at_modifier_display_start() {
        assert_eq!(AtModifier::Start.to_string(), "@ start()");
    }

    #[test]
    fn test_at_modifier_display_end() {
        assert_eq!(AtModifier::End.to_string(), "@ end()");
    }

    #[test]
    fn test_vector_selector_with_at() {
        let (rest, sel) = vector_selector("foo @ 1603774568").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.name, Some("foo".to_string()));
        assert_eq!(sel.at, Some(AtModifier::Timestamp(1_603_774_568_000)));
    }

    #[test]
    fn test_vector_selector_with_at_start() {
        let (rest, sel) = vector_selector("foo @ start()").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.at, Some(AtModifier::Start));
    }

    #[test]
    fn test_vector_selector_with_at_and_offset() {
        // @ before offset
        let (rest, sel) = vector_selector("foo @ 123 offset 5m").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.at, Some(AtModifier::Timestamp(123_000)));
        assert_eq!(sel.offset.unwrap().as_millis(), 5 * 60 * 1000);
    }

    #[test]
    fn test_vector_selector_with_offset_and_at() {
        // offset before @
        let (rest, sel) = vector_selector("foo offset 5m @ 123").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.at, Some(AtModifier::Timestamp(123_000)));
        assert_eq!(sel.offset.unwrap().as_millis(), 5 * 60 * 1000);
    }

    #[test]
    fn test_vector_selector_display_with_at() {
        let mut sel = VectorSelector::new("foo");
        sel.at = Some(AtModifier::Timestamp(123_000));
        assert_eq!(sel.to_string(), "foo @ 123.000");
    }

    #[test]
    fn test_vector_selector_display_with_at_and_offset() {
        let mut sel = VectorSelector::new("foo");
        sel.at = Some(AtModifier::Start);
        sel.offset = Some(Duration::from_secs(300));
        assert_eq!(sel.to_string(), "foo @ start() offset 5m");
    }

    #[test]
    fn test_matrix_selector_with_at() {
        let (rest, sel) = matrix_selector("foo[5m] @ 123").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.at(), Some(&AtModifier::Timestamp(123_000)));
    }

    #[test]
    fn test_matrix_selector_with_at_and_offset() {
        let (rest, sel) = matrix_selector("foo[5m] @ 123 offset 1h").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.at(), Some(&AtModifier::Timestamp(123_000)));
        assert_eq!(sel.offset_millis(), Some(60 * 60 * 1000));
    }

    #[test]
    fn test_matrix_selector_with_offset_and_at() {
        let (rest, sel) = matrix_selector("foo[5m] offset 1h @ 123").unwrap();
        assert!(rest.is_empty());
        assert_eq!(sel.at(), Some(&AtModifier::Timestamp(123_000)));
        assert_eq!(sel.offset_millis(), Some(60 * 60 * 1000));
    }

    #[test]
    fn test_matrix_selector_display_with_at() {
        let mut vs = VectorSelector::new("foo");
        vs.at = Some(AtModifier::Start);
        let sel = MatrixSelector::new(vs, Duration::from_secs(300));
        assert_eq!(sel.to_string(), "foo[5m] @ start()");
    }

    #[test]
    fn test_matrix_selector_display_with_at_and_offset() {
        let mut vs = VectorSelector::new("foo");
        vs.at = Some(AtModifier::Start);
        vs.offset = Some(Duration::from_secs(60));
        let sel = MatrixSelector::new(vs, Duration::from_secs(300));
        assert_eq!(sel.to_string(), "foo[5m] @ start() offset 1m");
    }
}
