//! Whitespace and comment parsers for PromQL.
//!
//! This module provides parsers for handling whitespace and comments
//! in PromQL expressions. Whitespace includes spaces, tabs, newlines,
//! and carriage returns. Comments start with `#` and extend to end of line.
//!
//! # Examples
//!
//! ```rust
//! use rusty_promql_parser::lexer::whitespace::{ws_opt, line_comment};
//!
//! // Skip optional whitespace
//! let (rest, _) = ws_opt("  \n  foo").unwrap();
//! assert_eq!(rest, "foo");
//!
//! // Skip whitespace and comments
//! let (rest, _) = ws_opt("# comment\nfoo").unwrap();
//! assert_eq!(rest, "foo");
//! ```

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::{char, not_line_ending},
    combinator::value,
    multi::many0,
    sequence::preceded,
};

/// Check if a character is whitespace (space, tab, newline, or carriage return).
#[inline]
pub fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n' || c == '\r'
}

/// Parse zero or more whitespace characters (spaces, tabs, newlines).
/// Does NOT consume comments.
pub fn whitespace0(input: &str) -> IResult<&str, &str> {
    take_while(is_whitespace)(input)
}

/// Parse one or more whitespace characters (spaces, tabs, newlines).
/// Does NOT consume comments.
pub fn whitespace1(input: &str) -> IResult<&str, &str> {
    take_while1(is_whitespace)(input)
}

/// Parse a line comment starting with '#'.
/// Consumes the '#' and all characters until (but not including) the end of line.
/// Returns the comment content (without the '#' prefix).
pub fn line_comment(input: &str) -> IResult<&str, &str> {
    preceded(char('#'), not_line_ending).parse(input)
}

/// Parse a single whitespace element: either whitespace characters or a comment.
fn ws_element(input: &str) -> IResult<&str, ()> {
    alt((value((), whitespace1), value((), line_comment))).parse(input)
}

/// Parse optional whitespace and comments (ws_opt in the pest grammar).
/// This is the most commonly used whitespace parser - it consumes any combination
/// of whitespace and comments, returning the empty unit.
///
/// Use this between tokens where whitespace is optional.
pub fn ws_opt(input: &str) -> IResult<&str, ()> {
    value((), many0(ws_element)).parse(input)
}

/// Parse required whitespace and/or comments (ws_req in the pest grammar).
/// At least one whitespace character or comment must be present.
///
/// Use this where whitespace is required (e.g., between `offset` and the duration).
pub fn ws_req(input: &str) -> IResult<&str, ()> {
    let (input, _) = ws_element(input)?;
    ws_opt(input)
}

/// Wrapper combinator that consumes optional leading whitespace before a parser.
///
/// Example:
/// ```
/// use rusty_promql_parser::lexer::ws;
/// use nom::bytes::complete::tag;
///
/// let mut parser = ws(tag("foo"));
/// assert_eq!(parser("  foo"), Ok(("", "foo")));
/// ```
pub fn ws<'a, O, F>(mut parser: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
{
    move |input: &'a str| {
        let (input, _) = ws_opt(input)?;
        parser.parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_whitespace() {
        assert!(is_whitespace(' '));
        assert!(is_whitespace('\t'));
        assert!(is_whitespace('\n'));
        assert!(is_whitespace('\r'));
        assert!(!is_whitespace('a'));
        assert!(!is_whitespace('#'));
    }

    #[test]
    fn test_whitespace0() {
        assert_eq!(whitespace0(""), Ok(("", "")));
        assert_eq!(whitespace0("abc"), Ok(("abc", "")));
        assert_eq!(whitespace0("   "), Ok(("", "   ")));
        assert_eq!(whitespace0("  abc"), Ok(("abc", "  ")));
        assert_eq!(whitespace0("\t\n  "), Ok(("", "\t\n  ")));
        assert_eq!(whitespace0("\r\n"), Ok(("", "\r\n")));
    }

    #[test]
    fn test_whitespace1() {
        assert!(whitespace1("").is_err());
        assert!(whitespace1("abc").is_err());
        assert_eq!(whitespace1("   "), Ok(("", "   ")));
        assert_eq!(whitespace1("  abc"), Ok(("abc", "  ")));
        assert_eq!(whitespace1("\t\n  x"), Ok(("x", "\t\n  ")));
    }

    #[test]
    fn test_line_comment() {
        assert_eq!(
            line_comment("# this is a comment"),
            Ok(("", " this is a comment"))
        );
        assert_eq!(line_comment("#comment"), Ok(("", "comment")));
        assert_eq!(line_comment("# comment\nfoo"), Ok(("\nfoo", " comment")));
        assert_eq!(line_comment("#"), Ok(("", "")));
        assert!(line_comment("not a comment").is_err());
    }

    #[test]
    fn test_ws_opt_empty() {
        let (remaining, _) = ws_opt("").unwrap();
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_ws_opt_no_whitespace() {
        let (remaining, _) = ws_opt("foo").unwrap();
        assert_eq!(remaining, "foo");
    }

    #[test]
    fn test_ws_opt_spaces() {
        let (remaining, _) = ws_opt("   foo").unwrap();
        assert_eq!(remaining, "foo");
    }

    #[test]
    fn test_ws_opt_mixed() {
        let (remaining, _) = ws_opt("  \t\n  foo").unwrap();
        assert_eq!(remaining, "foo");
    }

    #[test]
    fn test_ws_opt_comment_only() {
        let (remaining, _) = ws_opt("# comment\nfoo").unwrap();
        assert_eq!(remaining, "foo");
    }

    #[test]
    fn test_ws_opt_whitespace_and_comment() {
        let (remaining, _) = ws_opt("  # comment\n  foo").unwrap();
        assert_eq!(remaining, "foo");
    }

    #[test]
    fn test_ws_opt_multiple_comments() {
        let (remaining, _) = ws_opt("# comment 1\n# comment 2\nfoo").unwrap();
        assert_eq!(remaining, "foo");
    }

    #[test]
    fn test_ws_opt_comment_at_end() {
        // Comment without trailing newline
        let (remaining, _) = ws_opt("# comment").unwrap();
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_ws_req_spaces() {
        let result = ws_req("   foo");
        assert!(result.is_ok());
        let (remaining, _) = result.unwrap();
        assert_eq!(remaining, "foo");
    }

    #[test]
    fn test_ws_req_comment() {
        let result = ws_req("# comment\nfoo");
        assert!(result.is_ok());
        let (remaining, _) = result.unwrap();
        assert_eq!(remaining, "foo");
    }

    #[test]
    fn test_ws_req_fails_on_no_whitespace() {
        let result = ws_req("foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_ws_combinator() {
        use nom::bytes::complete::tag;

        let mut parser = ws(tag("foo"));

        assert_eq!(parser("foo"), Ok(("", "foo")));
        assert_eq!(parser("  foo"), Ok(("", "foo")));
        assert_eq!(parser("\n\tfoo"), Ok(("", "foo")));
        assert_eq!(parser("# comment\nfoo"), Ok(("", "foo")));
        assert_eq!(parser("  # comment\n  foo"), Ok(("", "foo")));
    }

    #[test]
    fn test_real_promql_whitespace() {
        // Test with realistic PromQL-like input
        let (remaining, _) = ws_opt("  \n  # Select all http requests\n  ").unwrap();
        assert_eq!(remaining, "");

        let (remaining, _) = ws_opt("   http_requests").unwrap();
        assert_eq!(remaining, "http_requests");
    }
}
