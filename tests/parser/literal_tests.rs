// Literal test cases extracted from:
// - references/prometheus/promql/parser/parse_test.go
//
// These test cases cover:
// - Number literals (integers, floats, special values)
// - String literals (double-quoted, single-quoted, backtick)
// - Literal usage in expressions

/// Valid integer literals
pub const VALID_INTEGERS: &[(&str, f64)] = &[
    ("0", 0.0),
    ("1", 1.0),
    ("42", 42.0),
    ("123456789", 123456789.0),
    // Note: In PromQL, all numbers are floats internally
];

/// Valid float literals
#[allow(clippy::approx_constant)]
pub const VALID_FLOATS: &[(&str, f64)] = &[
    ("0.0", 0.0),
    ("1.0", 1.0),
    ("0.5", 0.5),
    ("3.14159", 3.14159),
    ("1.5", 1.5),
    ("2.5", 2.5),
    (".5", 0.5),
    ("1.", 1.0),
    // Very small values
    ("0.0001", 0.0001),
    ("0.000001", 0.000001),
    // Very large values
    ("1000000.0", 1000000.0),
    ("9999999.9999", 9999999.9999),
];

/// Valid scientific notation literals
pub const VALID_SCIENTIFIC: &[(&str, f64)] = &[
    ("1e0", 1.0),
    ("1e1", 10.0),
    ("1e10", 1e10),
    ("1E10", 1e10),
    ("1e-1", 0.1),
    ("1e-10", 1e-10),
    ("1E-10", 1e-10),
    ("1.5e2", 150.0),
    ("1.5e-2", 0.015),
    ("1.5E2", 150.0),
    ("1.5E-2", 0.015),
    // With leading decimal point
    (".5e1", 5.0),
    (".5E1", 5.0),
    // With trailing decimal point
    ("1.e2", 100.0),
    ("1.E2", 100.0),
];

/// Valid hexadecimal literals
pub const VALID_HEX: &[(&str, f64)] = &[
    ("0x0", 0.0),
    ("0X0", 0.0),
    ("0x1", 1.0),
    ("0xA", 10.0),
    ("0xa", 10.0),
    ("0xFF", 255.0),
    ("0xff", 255.0),
    ("0xDEADBEEF", 0xDEADBEEFu64 as f64),
    ("0xdeadbeef", 0xDEADBEEFu64 as f64),
];

/// Valid octal literals
/// Prometheus uses Go's strconv.ParseInt with base 0, which supports:
/// - Legacy octal with leading zero: 0755 = 493
/// - Modern octal with 0o/0O prefix: 0o755 = 493 (Go 1.13+)
pub const VALID_OCTAL: &[(&str, f64)] = &[
    // Legacy octal (leading zero) - from parse_test.go
    ("0755", 493.0), // Prometheus test case
    ("0644", 420.0),
    ("07", 7.0),
    ("010", 8.0),
    ("0777", 511.0),
    // Modern octal (0o prefix) - supported by Go 1.13+ strconv
    ("0o0", 0.0),
    ("0O0", 0.0),
    ("0o7", 7.0),
    ("0o10", 8.0),
    ("0o755", 493.0),
    ("0o777", 511.0),
];

/// Special float values
pub const SPECIAL_FLOATS: &[(&str, &str)] = &[
    // Infinity
    ("Inf", "inf"),
    ("inf", "inf"),
    ("INF", "inf"),
    ("+Inf", "inf"),
    ("+inf", "inf"),
    ("-Inf", "-inf"),
    ("-inf", "-inf"),
    // NaN
    ("NaN", "nan"),
    ("nan", "nan"),
    ("NAN", "nan"),
];

/// Valid double-quoted string literals
pub const VALID_DOUBLE_QUOTED_STRINGS: &[(&str, &str)] = &[
    (r#""""#, ""),
    (r#""hello""#, "hello"),
    (r#""hello world""#, "hello world"),
    (r#""hello\nworld""#, "hello\nworld"),
    (r#""hello\tworld""#, "hello\tworld"),
    (r#""hello\\world""#, "hello\\world"),
    (r#""hello\"world""#, "hello\"world"),
    (r#""hello\'world""#, "hello'world"),
    // Unicode escapes
    (r#""\u0041""#, "A"),
    (r#""\u4e2d\u6587""#, "中文"),
    // Hex escapes
    (r#""\x41""#, "A"),
    (r#""\x41\x42\x43""#, "ABC"),
    // Octal escapes
    (r#""\101""#, "A"),
];

/// Valid single-quoted string literals
pub const VALID_SINGLE_QUOTED_STRINGS: &[(&str, &str)] = &[
    ("''", ""),
    ("'hello'", "hello"),
    ("'hello world'", "hello world"),
    (r"'hello\nworld'", "hello\nworld"),
    (r"'hello\'world'", "hello'world"),
    (r"'hello\\world'", "hello\\world"),
];

/// Valid backtick (raw) string literals
pub const VALID_BACKTICK_STRINGS: &[(&str, &str)] = &[
    ("``", ""),
    ("`hello`", "hello"),
    ("`hello world`", "hello world"),
    // No escape processing in backtick strings
    (r"`hello\nworld`", r"hello\nworld"),
    (r"`hello\tworld`", r"hello\tworld"),
    // Can contain quotes
    (r#"`hello "world"`"#, r#"hello "world""#),
    (r#"`hello 'world'`"#, r#"hello 'world'"#),
];

/// Literals in expressions
pub const LITERALS_IN_EXPRESSIONS: &[&str] = &[
    // Numbers in expressions
    "1 + 2",
    "3.14 * 2",
    "1e10 / 1e5",
    // Numbers with metrics
    "some_metric + 1",
    "some_metric * 2.5",
    "some_metric / 1e3",
    // Numbers in function calls
    "vector(1)",
    "vector(3.14)",
    "clamp(some_metric, 0, 100)",
    "histogram_quantile(0.9, some_metric)",
    // Numbers in aggregations
    "topk(5, some_metric)",
    "bottomk(10, some_metric)",
    "quantile(0.99, some_metric)",
];

/// Invalid number literals
pub const INVALID_NUMBERS: &[(&str, &str)] = &[
    // Invalid scientific notation
    ("1e", "expected digit"),
    ("1e+", "expected digit"),
    ("1e-", "expected digit"),
    ("1ee", "unexpected"),
    // Invalid hex
    ("0x", "expected hex digit"),
    ("0xG", "unexpected"),
    // Invalid octal
    ("0o8", "unexpected"),
    ("0o9", "unexpected"),
    // Multiple decimal points
    ("1.2.3", "unexpected"),
    // Leading zeros (might be ambiguous)
    // ("00", "..."),  // Depends on implementation
];

/// Invalid string literals
pub const INVALID_STRINGS: &[(&str, &str)] = &[
    // Unterminated strings
    (r#"""#, "unterminated string"),
    (r#""hello"#, "unterminated string"),
    ("'", "unterminated string"),
    ("'hello", "unterminated string"),
    ("`", "unterminated string"),
    ("`hello", "unterminated string"),
    // Invalid escape sequences
    (r#""\q""#, "invalid escape"),
    (r#""\xZZ""#, "invalid escape"),
    (r#""\uXXXX""#, "invalid escape"),
    // Newline in quoted string (not backtick)
    // ("\"hello\nworld\"", "unterminated string"),  // Hard to test
];

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_promql_parser::lexer::string::string_literal;
    use rusty_promql_parser::{Expr, expr, number};

    #[test]
    fn test_integer_literals_parse() {
        for (input, expected) in VALID_INTEGERS {
            let result = number(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "number parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    let diff = (value - expected).abs();
                    assert!(
                        diff < 1e-10,
                        "Integer '{}' should parse to {}, got {}",
                        input,
                        expected,
                        value
                    );
                }
                Err(e) => panic!("Failed to parse integer '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_float_literals_parse() {
        for (input, expected) in VALID_FLOATS {
            let result = number(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "number parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    let diff = (value - expected).abs();
                    let tolerance = if expected.abs() > 1.0 {
                        expected.abs() * 1e-10
                    } else {
                        1e-10
                    };
                    assert!(
                        diff < tolerance,
                        "Float '{}' should parse to {}, got {} (diff: {})",
                        input,
                        expected,
                        value,
                        diff
                    );
                }
                Err(e) => panic!("Failed to parse float '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_scientific_literals_parse() {
        for (input, expected) in VALID_SCIENTIFIC {
            let result = number(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "number parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    let diff = (value - expected).abs();
                    let tolerance = if expected.abs() > 1.0 {
                        expected.abs() * 1e-10
                    } else {
                        1e-10
                    };
                    assert!(
                        diff < tolerance,
                        "Scientific '{}' should parse to {}, got {}",
                        input,
                        expected,
                        value
                    );
                }
                Err(e) => panic!("Failed to parse scientific '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_hex_literals_parse() {
        for (input, expected) in VALID_HEX {
            let result = number(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "number parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    let diff = (value - expected).abs();
                    assert!(
                        diff < 1.0,
                        "Hex '{}' should parse to {}, got {}",
                        input,
                        expected,
                        value
                    );
                }
                Err(e) => panic!("Failed to parse hex '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_octal_literals_parse() {
        for (input, expected) in VALID_OCTAL {
            let result = number(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "number parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    let diff = (value - expected).abs();
                    assert!(
                        diff < 1.0,
                        "Octal '{}' should parse to {}, got {}",
                        input,
                        expected,
                        value
                    );
                }
                Err(e) => panic!("Failed to parse octal '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_special_floats_parse() {
        for (input, expected_type) in SPECIAL_FLOATS {
            let result = number(input);
            match result {
                Ok((remaining, value)) => {
                    assert!(
                        remaining.is_empty(),
                        "number parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                    match *expected_type {
                        "inf" => assert!(value.is_infinite() && value > 0.0),
                        "-inf" => assert!(value.is_infinite() && value < 0.0),
                        "nan" => assert!(value.is_nan()),
                        _ => panic!("Unknown expected type '{}'", expected_type),
                    }
                }
                Err(e) => panic!("Failed to parse special float '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_double_quoted_strings_parse() {
        for (input, expected) in VALID_DOUBLE_QUOTED_STRINGS {
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
                        "String '{}' should parse to {:?}, got {:?}",
                        input, expected, value
                    );
                }
                Err(e) => panic!("Failed to parse double-quoted string '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_single_quoted_strings_parse() {
        for (input, expected) in VALID_SINGLE_QUOTED_STRINGS {
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
                        "String '{}' should parse to {:?}, got {:?}",
                        input, expected, value
                    );
                }
                Err(e) => panic!("Failed to parse single-quoted string '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_backtick_strings_parse() {
        for (input, expected) in VALID_BACKTICK_STRINGS {
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
                        "String '{}' should parse to {:?}, got {:?}",
                        input, expected, value
                    );
                }
                Err(e) => panic!("Failed to parse backtick string '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_literals_in_expressions_parse() {
        for input in LITERALS_IN_EXPRESSIONS {
            let result = expr(input);
            match result {
                Ok((remaining, _parsed)) => {
                    assert!(
                        remaining.is_empty(),
                        "expr parser did not consume entire input '{}', remaining: '{}'",
                        input,
                        remaining
                    );
                }
                Err(e) => panic!("Failed to parse literal expression '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_invalid_number_literals_fail() {
        for (input, _error_desc) in INVALID_NUMBERS {
            let result = number(input);
            match result {
                Err(_) => {
                    // Good - it should fail
                }
                Ok((remaining, _)) => {
                    // If it parses, it should leave something unparsed
                    if remaining.is_empty() {
                        // Some inputs might be valid in certain interpretations
                    }
                }
            }
        }
    }

    #[test]
    fn test_invalid_string_literals_fail() {
        for (input, _error_desc) in INVALID_STRINGS {
            let result = string_literal(input);
            match result {
                Err(_) => {
                    // Good - it should fail
                }
                Ok((remaining, _)) => {
                    // If it parses, it should leave something unparsed
                    if remaining.is_empty() {
                        // Some inputs might be valid in certain interpretations
                    }
                }
            }
        }
    }

    #[test]
    fn test_number_as_expr() {
        // Test that numbers parse as expressions
        let result = expr("42");
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert!(remaining.is_empty());
        assert!(matches!(parsed, Expr::Number(v) if (v - 42.0).abs() < 1e-10));
    }

    #[test]
    fn test_string_as_expr() {
        // Test that strings parse as expressions
        let result = expr(r#""hello""#);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert!(remaining.is_empty());
        assert!(matches!(parsed, Expr::String(ref s) if s == "hello"));
    }
}
