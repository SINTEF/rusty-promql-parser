//! Identifier parsing for PromQL
//!
//! PromQL has several types of identifiers:
//! - **Label names**: `[a-zA-Z_][a-zA-Z0-9_]*` - no colons allowed
//! - **Metric names**: `[a-zA-Z_:][a-zA-Z0-9_:]*` - colons allowed (for recording rules)
//!
//! Keywords in PromQL are context-sensitive - they can be used as metric names
//! or label names when not in a keyword position.

use nom::{
    IResult, Parser,
    bytes::complete::{take_while, take_while1},
    combinator::{recognize, verify},
    sequence::pair,
};

/// Result of parsing an identifier - distinguishes between regular identifiers
/// and metric identifiers (which contain colons)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identifier {
    /// Regular identifier (no colons) - can be used as label name or metric name
    Plain(String),
    /// Metric identifier (contains colons) - only valid as metric name
    Metric(String),
}

impl Identifier {
    /// Get the identifier value as a string slice
    pub fn as_str(&self) -> &str {
        match self {
            Identifier::Plain(s) => s,
            Identifier::Metric(s) => s,
        }
    }

    /// Check if this identifier contains a colon (metric identifier)
    pub fn has_colon(&self) -> bool {
        matches!(self, Identifier::Metric(_))
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        match self {
            Identifier::Plain(s) => s,
            Identifier::Metric(s) => s,
        }
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Check if a character is alphabetic or underscore (can start identifier)
#[inline]
fn is_alpha(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic()
}

/// Check if a character is alphanumeric or underscore (can continue identifier)
#[inline]
fn is_alpha_numeric(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric()
}

/// Check if a character can start a metric identifier (alpha, underscore, or colon)
#[inline]
fn is_metric_start(c: char) -> bool {
    c == '_' || c == ':' || c.is_ascii_alphabetic()
}

/// Check if a character can continue a metric identifier (alphanumeric, underscore, or colon)
#[inline]
fn is_metric_char(c: char) -> bool {
    c == '_' || c == ':' || c.is_ascii_alphanumeric()
}

/// Parse a label name: `[a-zA-Z_][a-zA-Z0-9_]*`
///
/// Label names cannot contain colons. This is used for label names in
/// label matchers like `{job="prometheus"}`.
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::lexer::identifier::label_name;
///
/// let (rest, name) = label_name("job").unwrap();
/// assert_eq!(name, "job");
/// assert!(rest.is_empty());
///
/// let (rest, name) = label_name("__name__").unwrap();
/// assert_eq!(name, "__name__");
/// ```
pub fn label_name(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        verify(take_while1(is_alpha), |s: &str| {
            s.chars().next().is_some_and(is_alpha)
        }),
        take_while(is_alpha_numeric),
    ))
    .parse(input)
}

/// Parse a metric name: `[a-zA-Z_:][a-zA-Z0-9_:]*`
///
/// Metric names can contain colons (for recording rules).
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::lexer::identifier::metric_name;
///
/// let (rest, name) = metric_name("http_requests_total").unwrap();
/// assert_eq!(name, "http_requests_total");
///
/// let (rest, name) = metric_name("job:request_rate:5m").unwrap();
/// assert_eq!(name, "job:request_rate:5m");
///
/// // Can start with colon
/// let (rest, name) = metric_name(":request_rate").unwrap();
/// assert_eq!(name, ":request_rate");
/// ```
pub fn metric_name(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        verify(take_while1(is_metric_start), |s: &str| {
            s.chars().next().is_some_and(is_metric_start)
        }),
        take_while(is_metric_char),
    ))
    .parse(input)
}

/// Parse an identifier (either label name or metric identifier)
///
/// Returns `Identifier::Plain` for identifiers without colons,
/// and `Identifier::Metric` for identifiers with colons.
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::lexer::identifier::{identifier, Identifier};
///
/// let (_, id) = identifier("foo").unwrap();
/// assert_eq!(id, Identifier::Plain("foo".to_string()));
///
/// let (_, id) = identifier("foo:bar").unwrap();
/// assert_eq!(id, Identifier::Metric("foo:bar".to_string()));
/// ```
pub fn identifier(input: &str) -> IResult<&str, Identifier> {
    let (rest, name) = metric_name(input)?;
    let ident = if name.contains(':') {
        Identifier::Metric(name.to_string())
    } else {
        Identifier::Plain(name.to_string())
    };
    Ok((rest, ident))
}

/// PromQL keywords
///
/// These keywords have special meaning in certain contexts but can also
/// be used as identifiers (metric names, label names) in other contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    // Aggregation operators
    Sum,
    Avg,
    Count,
    Min,
    Max,
    Group,
    Stddev,
    Stdvar,
    Topk,
    Bottomk,
    CountValues,
    Quantile,
    Limitk,
    LimitRatio,

    // Set operators
    And,
    Or,
    Unless,

    // Binary operator
    Atan2,

    // Modifiers
    Offset,
    By,
    Without,
    On,
    Ignoring,
    GroupLeft,
    GroupRight,
    Bool,

    // @ modifier preprocessors
    Start,
    End,
    Step,
}

impl Keyword {
    /// Get the keyword as a string slice (lowercase)
    pub fn as_str(&self) -> &'static str {
        match self {
            Keyword::Sum => "sum",
            Keyword::Avg => "avg",
            Keyword::Count => "count",
            Keyword::Min => "min",
            Keyword::Max => "max",
            Keyword::Group => "group",
            Keyword::Stddev => "stddev",
            Keyword::Stdvar => "stdvar",
            Keyword::Topk => "topk",
            Keyword::Bottomk => "bottomk",
            Keyword::CountValues => "count_values",
            Keyword::Quantile => "quantile",
            Keyword::Limitk => "limitk",
            Keyword::LimitRatio => "limit_ratio",
            Keyword::And => "and",
            Keyword::Or => "or",
            Keyword::Unless => "unless",
            Keyword::Atan2 => "atan2",
            Keyword::Offset => "offset",
            Keyword::By => "by",
            Keyword::Without => "without",
            Keyword::On => "on",
            Keyword::Ignoring => "ignoring",
            Keyword::GroupLeft => "group_left",
            Keyword::GroupRight => "group_right",
            Keyword::Bool => "bool",
            Keyword::Start => "start",
            Keyword::End => "end",
            Keyword::Step => "step",
        }
    }

    /// Check if this keyword is an aggregation operator
    pub fn is_aggregation(&self) -> bool {
        matches!(
            self,
            Keyword::Sum
                | Keyword::Avg
                | Keyword::Count
                | Keyword::Min
                | Keyword::Max
                | Keyword::Group
                | Keyword::Stddev
                | Keyword::Stdvar
                | Keyword::Topk
                | Keyword::Bottomk
                | Keyword::CountValues
                | Keyword::Quantile
                | Keyword::Limitk
                | Keyword::LimitRatio
        )
    }

    /// Check if this keyword is an aggregation that takes a parameter
    pub fn is_aggregation_with_param(&self) -> bool {
        matches!(
            self,
            Keyword::Topk
                | Keyword::Bottomk
                | Keyword::CountValues
                | Keyword::Quantile
                | Keyword::Limitk
                | Keyword::LimitRatio
        )
    }

    /// Check if this keyword is a set operator
    pub fn is_set_operator(&self) -> bool {
        matches!(self, Keyword::And | Keyword::Or | Keyword::Unless)
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Try to look up a keyword from a string (case-insensitive)
fn lookup_keyword(s: &str) -> Option<Keyword> {
    match s.to_ascii_lowercase().as_str() {
        // Aggregation operators
        "sum" => Some(Keyword::Sum),
        "avg" => Some(Keyword::Avg),
        "count" => Some(Keyword::Count),
        "min" => Some(Keyword::Min),
        "max" => Some(Keyword::Max),
        "group" => Some(Keyword::Group),
        "stddev" => Some(Keyword::Stddev),
        "stdvar" => Some(Keyword::Stdvar),
        "topk" => Some(Keyword::Topk),
        "bottomk" => Some(Keyword::Bottomk),
        "count_values" => Some(Keyword::CountValues),
        "quantile" => Some(Keyword::Quantile),
        "limitk" => Some(Keyword::Limitk),
        "limit_ratio" => Some(Keyword::LimitRatio),
        // Set operators
        "and" => Some(Keyword::And),
        "or" => Some(Keyword::Or),
        "unless" => Some(Keyword::Unless),
        // Binary operator
        "atan2" => Some(Keyword::Atan2),
        // Modifiers
        "offset" => Some(Keyword::Offset),
        "by" => Some(Keyword::By),
        "without" => Some(Keyword::Without),
        "on" => Some(Keyword::On),
        "ignoring" => Some(Keyword::Ignoring),
        "group_left" => Some(Keyword::GroupLeft),
        "group_right" => Some(Keyword::GroupRight),
        "bool" => Some(Keyword::Bool),
        // @ modifier preprocessors
        "start" => Some(Keyword::Start),
        "end" => Some(Keyword::End),
        "step" => Some(Keyword::Step),
        _ => None,
    }
}

/// Parse a keyword (case-insensitive)
///
/// Keywords are recognized only when they are complete words (not followed
/// by alphanumeric characters).
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::lexer::identifier::{keyword, Keyword};
///
/// let (_, kw) = keyword("SUM").unwrap();
/// assert_eq!(kw, Keyword::Sum);
///
/// let (_, kw) = keyword("count_values").unwrap();
/// assert_eq!(kw, Keyword::CountValues);
/// ```
pub fn keyword(input: &str) -> IResult<&str, Keyword> {
    // Parse as identifier first (no colons for keywords)
    let (rest, word) = recognize(pair(
        verify(take_while1(is_alpha), |s: &str| {
            s.chars().next().is_some_and(is_alpha)
        }),
        take_while(is_alpha_numeric),
    ))
    .parse(input)?;

    // Check if it's a keyword
    if let Some(kw) = lookup_keyword(word) {
        Ok((rest, kw))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

/// Parse a keyword or identifier
///
/// This is the main entry point for lexing identifiers in PromQL.
/// It first tries to match a keyword, and if that fails, parses as identifier.
///
/// # Examples
///
/// ```
/// use rusty_promql_parser::lexer::identifier::{keyword_or_identifier, KeywordOrIdentifier, Keyword, Identifier};
///
/// // Keywords are recognized
/// let (_, result) = keyword_or_identifier("sum").unwrap();
/// assert_eq!(result, KeywordOrIdentifier::Keyword(Keyword::Sum));
///
/// // Regular identifiers
/// let (_, result) = keyword_or_identifier("http_requests").unwrap();
/// assert_eq!(result, KeywordOrIdentifier::Identifier(Identifier::Plain("http_requests".to_string())));
///
/// // Metric identifiers (with colon)
/// let (_, result) = keyword_or_identifier("job:rate:5m").unwrap();
/// assert_eq!(result, KeywordOrIdentifier::Identifier(Identifier::Metric("job:rate:5m".to_string())));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeywordOrIdentifier {
    Keyword(Keyword),
    Identifier(Identifier),
}

pub fn keyword_or_identifier(input: &str) -> IResult<&str, KeywordOrIdentifier> {
    // First try to parse as keyword
    if let Ok((rest, kw)) = keyword(input) {
        return Ok((rest, KeywordOrIdentifier::Keyword(kw)));
    }
    // Otherwise parse as identifier
    let (rest, id) = identifier(input)?;
    Ok((rest, KeywordOrIdentifier::Identifier(id)))
}

/// Try to parse a specific aggregation operator (case-insensitive)
pub fn aggregation_op(input: &str) -> IResult<&str, Keyword> {
    let (rest, kw) = keyword(input)?;
    if kw.is_aggregation() {
        Ok((rest, kw))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

/// Try to parse a set operator (and, or, unless) - case-insensitive
pub fn set_operator(input: &str) -> IResult<&str, Keyword> {
    let (rest, kw) = keyword(input)?;
    if kw.is_set_operator() {
        Ok((rest, kw))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Label name tests
    #[test]
    fn test_label_name_simple() {
        let (rest, name) = label_name("foo").unwrap();
        assert_eq!(name, "foo");
        assert!(rest.is_empty());
    }

    #[test]
    fn test_label_name_with_underscore() {
        let (rest, name) = label_name("some_label").unwrap();
        assert_eq!(name, "some_label");
        assert!(rest.is_empty());
    }

    #[test]
    fn test_label_name_starting_with_underscore() {
        let (rest, name) = label_name("_label").unwrap();
        assert_eq!(name, "_label");
        assert!(rest.is_empty());
    }

    #[test]
    fn test_label_name_reserved() {
        let (rest, name) = label_name("__name__").unwrap();
        assert_eq!(name, "__name__");
        assert!(rest.is_empty());
    }

    #[test]
    fn test_label_name_stops_at_colon() {
        // Label names don't include colons
        let (rest, name) = label_name("foo:bar").unwrap();
        assert_eq!(name, "foo");
        assert_eq!(rest, ":bar");
    }

    #[test]
    fn test_label_name_fails_on_number_start() {
        assert!(label_name("0foo").is_err());
    }

    // Metric name tests
    #[test]
    fn test_metric_name_simple() {
        let (rest, name) = metric_name("http_requests").unwrap();
        assert_eq!(name, "http_requests");
        assert!(rest.is_empty());
    }

    #[test]
    fn test_metric_name_with_colon() {
        let (rest, name) = metric_name("job:request_rate:5m").unwrap();
        assert_eq!(name, "job:request_rate:5m");
        assert!(rest.is_empty());
    }

    #[test]
    fn test_metric_name_starting_with_colon() {
        let (rest, name) = metric_name(":request_rate").unwrap();
        assert_eq!(name, ":request_rate");
        assert!(rest.is_empty());
    }

    #[test]
    fn test_metric_name_multiple_colons() {
        let (rest, name) = metric_name("a:b:c:d").unwrap();
        assert_eq!(name, "a:b:c:d");
        assert!(rest.is_empty());
    }

    #[test]
    fn test_metric_name_fails_on_number_start() {
        assert!(metric_name("0metric").is_err());
    }

    // Identifier tests
    #[test]
    fn test_identifier_plain() {
        let (rest, id) = identifier("foo").unwrap();
        assert_eq!(id, Identifier::Plain("foo".to_string()));
        assert!(rest.is_empty());
    }

    #[test]
    fn test_identifier_metric() {
        let (rest, id) = identifier("foo:bar").unwrap();
        assert_eq!(id, Identifier::Metric("foo:bar".to_string()));
        assert!(rest.is_empty());
    }

    #[test]
    fn test_identifier_has_colon() {
        let plain = Identifier::Plain("foo".to_string());
        let metric = Identifier::Metric("foo:bar".to_string());
        assert!(!plain.has_colon());
        assert!(metric.has_colon());
    }

    // Keyword tests
    #[test]
    fn test_keyword_sum() {
        let (rest, kw) = keyword("sum").unwrap();
        assert_eq!(kw, Keyword::Sum);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_keyword_case_insensitive() {
        let (_, kw1) = keyword("SUM").unwrap();
        let (_, kw2) = keyword("Sum").unwrap();
        let (_, kw3) = keyword("sum").unwrap();
        assert_eq!(kw1, Keyword::Sum);
        assert_eq!(kw2, Keyword::Sum);
        assert_eq!(kw3, Keyword::Sum);
    }

    #[test]
    fn test_keyword_count_values() {
        let (rest, kw) = keyword("count_values").unwrap();
        assert_eq!(kw, Keyword::CountValues);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_keyword_not_partial_match() {
        // "summary" should not match "sum"
        assert!(keyword("summary").is_err());
    }

    #[test]
    fn test_keyword_with_following_paren() {
        // "sum(" should match "sum" and leave "("
        let (rest, kw) = keyword("sum(").unwrap();
        assert_eq!(kw, Keyword::Sum);
        assert_eq!(rest, "(");
    }

    #[test]
    fn test_all_aggregation_keywords() {
        let aggregations = [
            ("sum", Keyword::Sum),
            ("avg", Keyword::Avg),
            ("count", Keyword::Count),
            ("min", Keyword::Min),
            ("max", Keyword::Max),
            ("group", Keyword::Group),
            ("stddev", Keyword::Stddev),
            ("stdvar", Keyword::Stdvar),
            ("topk", Keyword::Topk),
            ("bottomk", Keyword::Bottomk),
            ("count_values", Keyword::CountValues),
            ("quantile", Keyword::Quantile),
            ("limitk", Keyword::Limitk),
            ("limit_ratio", Keyword::LimitRatio),
        ];
        for (input, expected) in aggregations {
            let (_, kw) = keyword(input).unwrap();
            assert_eq!(kw, expected);
            assert!(kw.is_aggregation());
        }
    }

    #[test]
    fn test_set_operators() {
        let (_, kw) = keyword("and").unwrap();
        assert_eq!(kw, Keyword::And);
        assert!(kw.is_set_operator());

        let (_, kw) = keyword("or").unwrap();
        assert_eq!(kw, Keyword::Or);
        assert!(kw.is_set_operator());

        let (_, kw) = keyword("unless").unwrap();
        assert_eq!(kw, Keyword::Unless);
        assert!(kw.is_set_operator());
    }

    #[test]
    fn test_modifier_keywords() {
        let modifiers = [
            ("offset", Keyword::Offset),
            ("by", Keyword::By),
            ("without", Keyword::Without),
            ("on", Keyword::On),
            ("ignoring", Keyword::Ignoring),
            ("group_left", Keyword::GroupLeft),
            ("group_right", Keyword::GroupRight),
            ("bool", Keyword::Bool),
        ];
        for (input, expected) in modifiers {
            let (_, kw) = keyword(input).unwrap();
            assert_eq!(kw, expected);
        }
    }

    // keyword_or_identifier tests
    #[test]
    fn test_keyword_or_identifier_keyword() {
        let (_, result) = keyword_or_identifier("sum").unwrap();
        assert_eq!(result, KeywordOrIdentifier::Keyword(Keyword::Sum));
    }

    #[test]
    fn test_keyword_or_identifier_plain() {
        let (_, result) = keyword_or_identifier("http_requests").unwrap();
        assert_eq!(
            result,
            KeywordOrIdentifier::Identifier(Identifier::Plain("http_requests".to_string()))
        );
    }

    #[test]
    fn test_keyword_or_identifier_metric() {
        let (_, result) = keyword_or_identifier("job:rate:5m").unwrap();
        assert_eq!(
            result,
            KeywordOrIdentifier::Identifier(Identifier::Metric("job:rate:5m".to_string()))
        );
    }

    // aggregation_op tests
    #[test]
    fn test_aggregation_op() {
        let (_, kw) = aggregation_op("sum").unwrap();
        assert_eq!(kw, Keyword::Sum);
    }

    #[test]
    fn test_aggregation_op_rejects_non_aggregation() {
        assert!(aggregation_op("offset").is_err());
    }

    // set_operator tests
    #[test]
    fn test_set_operator_fn() {
        let (_, kw) = set_operator("and").unwrap();
        assert_eq!(kw, Keyword::And);
    }

    #[test]
    fn test_set_operator_rejects_non_set_op() {
        assert!(set_operator("sum").is_err());
    }

    // Edge cases
    #[test]
    fn test_nan_as_identifier() {
        // NaN starting an identifier should be parsed as identifier, not literal
        // But "NaN" alone is a float literal, handled by number parser
        // "NaN123" should be an identifier
        let (rest, id) = identifier("NaN123").unwrap();
        assert_eq!(id, Identifier::Plain("NaN123".to_string()));
        assert!(rest.is_empty());
    }

    #[test]
    fn test_inf_as_identifier() {
        // "Infoo" should be an identifier
        let (rest, id) = identifier("Infoo").unwrap();
        assert_eq!(id, Identifier::Plain("Infoo".to_string()));
        assert!(rest.is_empty());
    }

    #[test]
    fn test_keyword_as_part_of_identifier() {
        // "summary" contains "sum" but should parse as full identifier
        let (_, result) = keyword_or_identifier("summary").unwrap();
        assert_eq!(
            result,
            KeywordOrIdentifier::Identifier(Identifier::Plain("summary".to_string()))
        );
    }

    #[test]
    fn test_aggregation_with_param() {
        assert!(Keyword::Topk.is_aggregation_with_param());
        assert!(Keyword::Bottomk.is_aggregation_with_param());
        assert!(Keyword::CountValues.is_aggregation_with_param());
        assert!(Keyword::Quantile.is_aggregation_with_param());
        assert!(!Keyword::Sum.is_aggregation_with_param());
    }
}
