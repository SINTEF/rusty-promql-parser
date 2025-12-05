//! String literal parser for PromQL.
//!
//! PromQL supports three string literal formats:
//!
//! - **Double-quoted**: `"hello \"world\""`
//! - **Single-quoted**: `'hello \'world\''`
//! - **Raw/backtick**: `` `no escapes here` ``
//!
//! # Escape Sequences
//!
//! Double and single-quoted strings support these escape sequences:
//!
//! | Escape | Description        |
//! |--------|--------------------|
//! | `\a`   | Bell               |
//! | `\b`   | Backspace          |
//! | `\f`   | Form feed          |
//! | `\n`   | Newline            |
//! | `\r`   | Carriage return    |
//! | `\t`   | Tab                |
//! | `\v`   | Vertical tab       |
//! | `\\`   | Backslash          |
//! | `\"`   | Double quote       |
//! | `\'`   | Single quote       |
//! | `\xNN` | Hex (2 digits)     |
//! | `\NNN` | Octal (3 digits)   |
//! | `\uNNNN` | Unicode (4 hex)  |
//! | `\UNNNNNNNN` | Unicode (8 hex) |
//!
//! Raw strings (backtick) have no escape processing.
//!
//! # Examples
//!
//! ```rust
//! use rusty_promql_parser::lexer::string::string_literal;
//!
//! // Double-quoted
//! let (_, s) = string_literal(r#""hello""#).unwrap();
//! assert_eq!(s, "hello");
//!
//! // Single-quoted with escape
//! let (_, s) = string_literal(r"'line\nbreak'").unwrap();
//! assert_eq!(s, "line\nbreak");
//!
//! // Raw string (no escapes)
//! let (_, s) = string_literal(r"`\n is literal`").unwrap();
//! assert_eq!(s, r"\n is literal");
//! ```

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::take_while_m_n,
    character::complete::{anychar, char, none_of},
    combinator::{map, map_opt, value, verify},
    multi::many0,
    sequence::{delimited, preceded},
};

/// Parse a PromQL string literal and return the unescaped string value.
///
/// Accepts double-quoted, single-quoted, or backtick-quoted strings.
pub fn string_literal(input: &str) -> IResult<&str, String> {
    alt((double_quoted_string, single_quoted_string, raw_string)).parse(input)
}

/// Parse a double-quoted string: "hello \"world\""
pub fn double_quoted_string(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        map(many0(double_quoted_char), |chars| {
            chars.into_iter().collect()
        }),
        char('"'),
    )
    .parse(input)
}

/// Parse a single-quoted string: 'hello \'world\''
pub fn single_quoted_string(input: &str) -> IResult<&str, String> {
    delimited(
        char('\''),
        map(many0(single_quoted_char), |chars| {
            chars.into_iter().collect()
        }),
        char('\''),
    )
    .parse(input)
}

/// Parse a raw/backtick string: `no escapes`
/// In raw strings, backslashes are literal - no escape processing.
pub fn raw_string(input: &str) -> IResult<&str, String> {
    delimited(
        char('`'),
        map(many0(none_of("`")), |chars| chars.into_iter().collect()),
        char('`'),
    )
    .parse(input)
}

/// Parse a character inside a double-quoted string
fn double_quoted_char(input: &str) -> IResult<&str, char> {
    alt((
        // Escape sequence
        preceded(char('\\'), escape_char('"')),
        // Any char except quote, backslash, or newline
        verify(anychar, |&c| c != '"' && c != '\\' && c != '\n'),
    ))
    .parse(input)
}

/// Parse a character inside a single-quoted string
fn single_quoted_char(input: &str) -> IResult<&str, char> {
    alt((
        // Escape sequence
        preceded(char('\\'), escape_char('\'')),
        // Any char except quote, backslash, or newline
        verify(anychar, |&c| c != '\'' && c != '\\' && c != '\n'),
    ))
    .parse(input)
}

/// Parse an escape sequence (after the backslash)
/// The `quote_char` parameter specifies which quote character can be escaped
fn escape_char(quote_char: char) -> impl FnMut(&str) -> IResult<&str, char> {
    move |input: &str| {
        alt((
            // Simple escape sequences
            value('\x07', char('a')),            // Bell
            value('\x08', char('b')),            // Backspace
            value('\x0c', char('f')),            // Form feed
            value('\n', char('n')),              // Newline
            value('\r', char('r')),              // Carriage return
            value('\t', char('t')),              // Tab
            value('\x0b', char('v')),            // Vertical tab
            value('\\', char('\\')),             // Backslash
            value(quote_char, char(quote_char)), // Quote character
            // Also allow escaping the other quote (for compatibility)
            value('"', char('"')),
            value('\'', char('\'')),
            // Hex escape: \xNN
            hex_escape,
            // Unicode escapes: \uNNNN and \UNNNNNNNN
            unicode_escape_short,
            unicode_escape_long,
            // Octal escape: \NNN (3 octal digits)
            octal_escape,
        ))
        .parse(input)
    }
}

/// Parse a hex escape sequence: \xNN (2 hex digits)
fn hex_escape(input: &str) -> IResult<&str, char> {
    preceded(
        char('x'),
        map_opt(
            take_while_m_n(2, 2, |c: char| c.is_ascii_hexdigit()),
            |hex: &str| {
                let val = u8::from_str_radix(hex, 16).ok()?;
                Some(val as char)
            },
        ),
    )
    .parse(input)
}

/// Parse a short unicode escape sequence: \uNNNN (4 hex digits)
fn unicode_escape_short(input: &str) -> IResult<&str, char> {
    preceded(
        char('u'),
        map_opt(
            take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit()),
            |hex: &str| {
                let val = u32::from_str_radix(hex, 16).ok()?;
                // Check for surrogate range (invalid)
                if (0xD800..0xE000).contains(&val) {
                    return None;
                }
                char::from_u32(val)
            },
        ),
    )
    .parse(input)
}

/// Parse a long unicode escape sequence: \UNNNNNNNN (8 hex digits)
fn unicode_escape_long(input: &str) -> IResult<&str, char> {
    preceded(
        char('U'),
        map_opt(
            take_while_m_n(8, 8, |c: char| c.is_ascii_hexdigit()),
            |hex: &str| {
                let val = u32::from_str_radix(hex, 16).ok()?;
                // Check for surrogate range (invalid)
                if (0xD800..0xE000).contains(&val) {
                    return None;
                }
                char::from_u32(val)
            },
        ),
    )
    .parse(input)
}

/// Parse an octal escape sequence: \NNN (1-3 octal digits starting with 0-7)
/// The Go implementation reads exactly 3 octal digits
fn octal_escape(input: &str) -> IResult<&str, char> {
    map_opt(
        take_while_m_n(3, 3, |c: char| c.is_ascii_digit() && c < '8'),
        |oct: &str| {
            let val = u8::from_str_radix(oct, 8).ok()?;
            Some(val as char)
        },
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to test string parsing
    fn assert_string(input: &str, expected: &str) {
        let result = string_literal(input);
        match result {
            Ok((remaining, value)) => {
                assert!(
                    remaining.is_empty(),
                    "Parser did not consume entire input '{}', remaining: '{}'",
                    input,
                    remaining
                );
                assert_eq!(
                    value, expected,
                    "For input '{}', expected {:?}, got {:?}",
                    input, expected, value
                );
            }
            Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
        }
    }

    /// Helper to test that input fails to parse
    fn assert_string_fails(input: &str) {
        let result = string_literal(input);
        assert!(
            result.is_err() || !result.unwrap().0.is_empty(),
            "Expected '{}' to fail or not fully parse",
            input
        );
    }

    // Double-quoted strings
    #[test]
    fn test_double_quoted_basic() {
        assert_string(r#""hello""#, "hello");
        assert_string(r#""world""#, "world");
        assert_string(r#""test string""#, "test string");
        assert_string(r#""""#, ""); // Empty string
    }

    #[test]
    fn test_double_quoted_escaped_quote() {
        assert_string(r#""say \"hello\"""#, "say \"hello\"");
    }

    #[test]
    fn test_double_quoted_simple_escapes() {
        assert_string(r#""\n""#, "\n");
        assert_string(r#""\t""#, "\t");
        assert_string(r#""\r""#, "\r");
        assert_string(r#""\\""#, "\\");
        assert_string(r#""\a""#, "\x07");
        assert_string(r#""\b""#, "\x08");
        assert_string(r#""\f""#, "\x0c");
        assert_string(r#""\v""#, "\x0b");
    }

    #[test]
    fn test_double_quoted_hex_escape() {
        assert_string(r#""\xFF""#, "\u{ff}");
        assert_string(r#""\x00""#, "\0");
        assert_string(r#""\x41""#, "A");
    }

    #[test]
    fn test_double_quoted_unicode_escape() {
        assert_string(r#""\u0041""#, "A");
        assert_string(r#""\u1234""#, "\u{1234}");
        assert_string(r#""\U00010111""#, "\u{10111}");
    }

    #[test]
    fn test_double_quoted_octal_escape() {
        assert_string(r#""\377""#, "\u{ff}");
        assert_string(r#""\000""#, "\0");
        assert_string(r#""\101""#, "A");
    }

    // Single-quoted strings
    #[test]
    fn test_single_quoted_basic() {
        assert_string("'hello'", "hello");
        assert_string("'world'", "world");
        assert_string("''", ""); // Empty string
    }

    #[test]
    fn test_single_quoted_escaped_quote() {
        assert_string(r"'say \'hello\''", "say 'hello'");
    }

    #[test]
    fn test_single_quoted_escapes() {
        assert_string(r"'\n'", "\n");
        assert_string(r"'\t'", "\t");
        assert_string(r"'\\'", "\\");
    }

    // Raw/backtick strings
    #[test]
    fn test_raw_string_basic() {
        assert_string("`hello`", "hello");
        assert_string("`test string`", "test string");
        assert_string("``", ""); // Empty string
    }

    #[test]
    fn test_raw_string_no_escapes() {
        // Backslashes are literal in raw strings
        assert_string(r"`\n\t\\`", r"\n\t\\");
        assert_string(r"`test\.expression`", r"test\.expression");
    }

    #[test]
    fn test_raw_string_can_contain_quotes() {
        assert_string(r#"`"hello"`"#, "\"hello\"");
        assert_string(r"`'hello'`", "'hello'");
    }

    // Complex strings from test data
    #[test]
    fn test_complex_escape_sequence() {
        assert_string(
            r#""\a\b\f\n\r\t\v\\\" - \xFF\377\u1234\U00010111""#,
            "\x07\x08\x0c\n\r\t\x0b\\\" - \u{ff}\u{ff}\u{1234}\u{10111}",
        );
    }

    // Error cases
    #[test]
    fn test_unterminated_double_quoted() {
        assert_string_fails(r#"""#);
        assert_string_fails(r#""hello"#);
    }

    #[test]
    fn test_unterminated_single_quoted() {
        assert_string_fails("'");
        assert_string_fails("'hello");
    }

    #[test]
    fn test_unterminated_raw_string() {
        assert_string_fails("`");
        assert_string_fails("`hello");
    }

    #[test]
    fn test_newline_in_quoted_string() {
        // Newlines not allowed in double/single quoted strings
        assert_string_fails("\"hello\nworld\"");
        assert_string_fails("'hello\nworld'");
    }

    #[test]
    fn test_raw_string_can_have_newlines() {
        // But raw strings can have newlines
        assert_string("`hello\nworld`", "hello\nworld");
    }

    // Partial parsing tests
    #[test]
    fn test_string_followed_by_other_content() {
        let (remaining, value) = string_literal(r#""hello" world"#).unwrap();
        assert_eq!(value, "hello");
        assert_eq!(remaining, " world");
    }
}
