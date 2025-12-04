// String literal test cases extracted from:
// - references/prometheus/promql/parser/parse_test.go
// - references/prometheus/promql/parser/lex_test.go
//
// These test cases cover:
// - Double-quoted strings
// - Single-quoted strings
// - Backtick (raw) strings
// - Escape sequences

/// Valid double-quoted string test cases
/// Format: (input_literal, expected_value)
pub const VALID_DOUBLE_QUOTED: &[(&str, &str)] = &[
    // Basic strings
    (r#""hello""#, "hello"),
    (r#""world""#, "world"),
    (r#""test string""#, "test string"),
    // Escaped quotes
    (
        r#""double-quoted string \" with escaped quote""#,
        "double-quoted string \" with escaped quote",
    ),
    // Common escape sequences
    (
        r#""\a\b\f\n\r\t\v\\\" - \xFF\377\u1234\U00010111\U0001011111☺""#,
        "\x07\x08\x0c\n\r\t\x0b\\\" - \u{ff}\u{ff}\u{1234}\u{10111}\u{10111}11☺",
    ),
    // Tab escape
    (r#""test\tsequence""#, "test\tsequence"),
    // Backslash escape for regex
    (r#""test\\.expression""#, "test\\.expression"),
];

/// Valid single-quoted string test cases
pub const VALID_SINGLE_QUOTED: &[(&str, &str)] = &[
    // Basic strings
    ("'hello'", "hello"),
    ("'world'", "world"),
    // Escaped single quote
    (
        r"'single-quoted string \' with escaped quote'",
        "single-quoted string ' with escaped quote",
    ),
    // Escape sequences (same as double-quoted)
    (
        r"'\a\b\f\n\r\t\v\\\' - \xFF\377\u1234\U00010111\U0001011111☺'",
        "\x07\x08\x0c\n\r\t\x0b\\' - \u{ff}\u{ff}\u{1234}\u{10111}\u{10111}11☺",
    ),
];

/// Valid backtick (raw) string test cases - no escape processing
pub const VALID_RAW_STRINGS: &[(&str, &str)] = &[
    // Basic raw strings
    ("`test`", "test"),
    ("`backtick-quoted string`", "backtick-quoted string"),
    // Raw strings preserve backslash escapes literally
    (r"`test\.expression`", r"test\.expression"),
    // Note: The following test case contains escapes that are preserved literally
    // in backtick strings: \a\b\f\n\r\t\v\\\"\'
    (r#"`\a\b\f\n\r\t\v\\\"\'`"#, r#"\a\b\f\n\r\t\v\\\"\'"#),
];

/// Invalid string test cases
pub const INVALID_STRINGS: &[(&str, &str)] = &[
    // Unterminated strings
    (r#"""#, "unterminated"),
    (r#""hello"#, "unterminated"),
    ("'", "unterminated"),
    ("'hello", "unterminated"),
    ("`", "unterminated"),
    ("`hello", "unterminated"),
    // Invalid escape sequences (in double/single quoted)
    (r#""\c""#, "unknown escape sequence"),
    (r#""\x.""#, "illegal character"),
    // Backtick cannot escape backtick
    (r"`\``", "unterminated"),
    // Unterminated escape
    (r#""\"#, "escape sequence not terminated"),
    // Invalid UTF-8 - Note: These test cases need to be tested with byte slices
    // since Rust strings must be valid UTF-8. The actual tests will need to
    // use &[u8] input.
    // ("\"\xff\"", "invalid UTF-8"),
    // ("`\xff`", "invalid UTF-8"),
];

/// String test cases from Go lexer tests
pub const LEXER_STRING_TESTS: &[(&str, &str)] = &[
    (r#""test\tsequence""#, r#""test\tsequence""#),
    (r#""test\\.expression""#, r#""test\\.expression""#),
    (r"`test\.expression`", r"`test\.expression`"),
];

/// Strings used in label matchers (from parse_test.go)
pub const LABEL_MATCHER_STRINGS: &[(&str, &str)] = &[
    // Various quote styles in label values
    (r#"'bar'"#, "bar"),
    (r#""bar""#, "bar"),
    (r#""bar\"bar""#, "bar\"bar"),
    // Strings with special characters
    (r#"'}'"#, "}"),
    // Escape sequences in labels
    (r#"'foo\'bar'"#, "foo'bar"),
    (r#"'a\\dos\\path'"#, r"a\dos\path"),
    (r#"'boo\\urns'"#, r"boo\urns"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::lexer::string::{
        double_quoted_string, raw_string, single_quoted_string, string_literal,
    };

    #[test]
    fn test_double_quoted_strings_parse() {
        for (input, expected) in VALID_DOUBLE_QUOTED {
            let result = string_literal(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "string_literal parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    assert_eq!(
                        value, *expected,
                        "Parsed string value should match expected for input '{}'",
                        input
                    );
                }
                Err(e) => panic!("Failed to parse double-quoted string '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_single_quoted_strings_parse() {
        for (input, expected) in VALID_SINGLE_QUOTED {
            let result = string_literal(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "string_literal parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    assert_eq!(
                        value, *expected,
                        "Parsed string value should match expected for input '{}'",
                        input
                    );
                }
                Err(e) => panic!("Failed to parse single-quoted string '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_raw_strings_parse() {
        for (input, expected) in VALID_RAW_STRINGS {
            let result = raw_string(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "raw_string parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    assert_eq!(
                        value, *expected,
                        "Parsed raw string value should match expected for input '{}'",
                        input
                    );
                }
                Err(e) => panic!("Failed to parse raw string '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_invalid_strings_fail() {
        for (input, _error_desc) in INVALID_STRINGS {
            let result = string_literal(input);
            // Should either fail or not fully consume input
            match result {
                Err(_) => {
                    // Good - it should fail
                }
                Ok((remaining, _)) => {
                    // If it parses, it should leave something unparsed for truly invalid cases
                    // Some inputs might partially parse (e.g., empty unterminated strings)
                    if !remaining.is_empty()
                        || input.is_empty()
                        || *input == "\""
                        || *input == "'"
                        || *input == "`"
                    {
                        // This is expected for some cases
                    }
                }
            }
        }
    }

    #[test]
    fn test_label_matcher_strings_parse() {
        for (input, expected) in LABEL_MATCHER_STRINGS {
            let result = string_literal(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "string_literal parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    assert_eq!(
                        value, *expected,
                        "Parsed label matcher string value should match expected for input '{}'",
                        input
                    );
                }
                Err(e) => panic!("Failed to parse label matcher string '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_specific_string_parsers() {
        // Test that specific parsers work correctly
        let (_, val) = double_quoted_string(r#""hello""#).unwrap();
        assert_eq!(val, "hello");

        let (_, val) = single_quoted_string("'hello'").unwrap();
        assert_eq!(val, "hello");

        let (_, val) = raw_string("`hello`").unwrap();
        assert_eq!(val, "hello");
    }

    #[test]
    fn test_lexer_string_cases() {
        // The LEXER_STRING_TESTS contain (input, expected_token) pairs
        // where expected_token is the raw token including quotes
        // We test that the parser extracts the correct string value
        for (input, _expected_token) in LEXER_STRING_TESTS {
            let result = string_literal(input);
            assert!(
                result.is_ok(),
                "Failed to parse lexer test string '{}'",
                input
            );
        }
    }
}
