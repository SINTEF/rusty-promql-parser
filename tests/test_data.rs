// Test data for rusty-promql-parser
//
// This crate contains test cases extracted from:
// - Official Prometheus PromQL parser (Go): references/prometheus/promql/parser/
// - HPE Rust PromQL parser: references/prometheus-parser-rs/
//
// Test data is organized by category:
// - lexer: Token-level tests (numbers, strings, durations, identifiers)
// - parser: Expression-level tests (selectors, operators, functions, etc.)
// - integration: Complete query tests
// - common: Shared test utilities and macros

mod common;
mod integration;
mod lexer;
mod parser;

// Re-export all test data for use by the parser implementation tests
pub use common::*;
pub use integration::*;
pub use lexer::*;
pub use parser::*;

#[cfg(test)]
mod smoke_tests {
    use super::*;

    /// Verify all test data arrays are populated and valid
    #[test]
    fn test_lexer_data_exists() {
        // Verify number tests
        for (input, expected) in lexer::number_tests::VALID_NUMBERS {
            assert!(!input.is_empty(), "Empty input in VALID_NUMBERS");
            assert!(expected.is_finite(), "Expected finite value");
        }
        // Verify string tests
        for (input, _expected) in lexer::string_tests::VALID_DOUBLE_QUOTED {
            assert!(
                input.starts_with('"'),
                "Double-quoted string should start with '\"'"
            );
        }
        // Verify duration tests
        for (input, expected_ms) in lexer::duration_tests::VALID_SIMPLE_DURATIONS {
            assert!(!input.is_empty(), "Empty input in VALID_SIMPLE_DURATIONS");
            assert!(*expected_ms > 0, "Expected positive milliseconds");
        }
        // Verify identifier tests
        for name in lexer::identifier_tests::VALID_METRIC_NAMES {
            assert!(!name.is_empty(), "Empty metric name");
        }
    }

    #[test]
    fn test_parser_data_exists() {
        // Verify selector tests
        for input in parser::selector_tests::VALID_VECTOR_SELECTORS {
            assert!(!input.is_empty(), "Empty selector");
        }
        // Verify binary operator tests
        for (input, _op) in parser::binary_tests::ARITHMETIC_OPERATORS {
            assert!(!input.is_empty(), "Empty binary expression");
        }
        // Verify function tests
        for input in parser::function_tests::VALID_FUNCTION_CALLS {
            assert!(input.contains('('), "Function call should contain '('");
        }
        // Verify aggregation tests
        for input in parser::aggregation_tests::VALID_AGGREGATIONS_SIMPLE {
            assert!(input.contains('('), "Aggregation should contain '('");
        }
        // Verify subquery tests
        for input in parser::subquery_tests::VALID_SIMPLE_SUBQUERIES {
            assert!(input.contains(':'), "Subquery should contain ':'");
        }
        // Verify unary tests
        for input in parser::unary_tests::VALID_UNARY_MINUS {
            assert!(input.contains('-'), "Unary minus should contain '-'");
        }
        // Verify matrix tests
        for input in parser::matrix_tests::VALID_MATRIX_SELECTORS {
            assert!(input.contains('['), "Matrix selector should contain '['");
        }
        // Verify literal tests
        for (input, expected) in parser::literal_tests::VALID_INTEGERS {
            assert!(!input.is_empty(), "Empty integer literal");
            assert!(expected.is_finite(), "Expected finite value");
        }
    }

    #[test]
    fn test_integration_data_exists() {
        // Verify real-world queries
        for query in integration::REAL_WORLD_QUERIES {
            assert!(!query.is_empty(), "Empty real-world query");
        }
        // Verify alert expressions
        for expr in integration::ALERT_EXPRESSIONS {
            assert!(!expr.is_empty(), "Empty alert expression");
        }
        // Verify edge cases
        for case in integration::EDGE_CASES {
            assert!(!case.is_empty(), "Empty edge case");
        }
    }

    /// Count total test cases to get an idea of coverage
    #[test]
    fn test_count_test_cases() {
        let mut total = 0;

        // Lexer tests
        total += lexer::number_tests::VALID_NUMBERS.len();
        total += lexer::number_tests::VALID_SPECIAL_FLOATS.len();
        total += lexer::number_tests::INVALID_NUMBERS.len();
        total += lexer::string_tests::VALID_DOUBLE_QUOTED.len();
        total += lexer::string_tests::VALID_SINGLE_QUOTED.len();
        total += lexer::string_tests::VALID_RAW_STRINGS.len();
        total += lexer::string_tests::INVALID_STRINGS.len();
        total += lexer::duration_tests::VALID_SIMPLE_DURATIONS.len();
        total += lexer::duration_tests::VALID_COMPOUND_DURATIONS.len();
        total += lexer::duration_tests::INVALID_DURATIONS.len();
        total += lexer::identifier_tests::VALID_METRIC_NAMES.len();
        total += lexer::identifier_tests::VALID_LABEL_NAMES.len();
        total += lexer::identifier_tests::KEYWORDS.len();

        // Parser tests
        total += parser::selector_tests::VALID_VECTOR_SELECTORS.len();
        total += parser::selector_tests::INVALID_VECTOR_SELECTORS.len();
        total += parser::binary_tests::ARITHMETIC_OPERATORS.len();
        total += parser::binary_tests::COMPARISON_OPERATORS.len();
        total += parser::binary_tests::SET_OPERATORS.len();
        total += parser::binary_tests::INVALID_BINARY_OPS.len();
        total += parser::function_tests::VALID_FUNCTION_CALLS.len();
        total += parser::function_tests::INVALID_FUNCTION_CALLS.len();
        total += parser::function_tests::FUNCTION_SIGNATURES.len();
        total += parser::aggregation_tests::VALID_AGGREGATIONS_SIMPLE.len();
        total += parser::aggregation_tests::VALID_AGGREGATIONS_BY.len();
        total += parser::aggregation_tests::INVALID_AGGREGATIONS.len();
        total += parser::subquery_tests::VALID_SIMPLE_SUBQUERIES.len();
        total += parser::subquery_tests::INVALID_SUBQUERIES.len();
        total += parser::unary_tests::VALID_UNARY_MINUS.len();
        total += parser::unary_tests::VALID_UNARY_PLUS.len();
        total += parser::matrix_tests::VALID_MATRIX_SELECTORS.len();
        total += parser::matrix_tests::INVALID_MATRIX_SELECTORS.len();
        total += parser::literal_tests::VALID_INTEGERS.len();
        total += parser::literal_tests::VALID_FLOATS.len();
        total += parser::literal_tests::INVALID_NUMBERS.len();

        // Integration tests
        total += integration::REAL_WORLD_QUERIES.len();
        total += integration::ALERT_EXPRESSIONS.len();
        total += integration::EDGE_CASES.len();

        // We should have a substantial number of test cases
        assert!(
            total > 300,
            "Expected at least 300 test cases, got {}",
            total
        );
        println!("Total test cases extracted: {}", total);
    }
}
