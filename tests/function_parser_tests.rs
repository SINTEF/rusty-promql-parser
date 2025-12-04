// Integration tests for function call parsing
//
// These tests verify that the expression parser correctly handles
// function calls from the Go Prometheus parser test cases.

mod parser {
    pub mod function_tests;
}

use parser::function_tests::{FUNCTION_SIGNATURES, VALID_FUNCTION_CALLS};
use rusty_promql_parser::parser::function::get_function;
use rusty_promql_parser::{Expr, expr};

/// Test that valid function calls parse successfully
#[test]
fn test_valid_function_calls_from_test_data() {
    for input in VALID_FUNCTION_CALLS {
        let result = expr(input);
        assert!(
            result.is_ok(),
            "Failed to parse function call: {:?}\nError: {:?}",
            input,
            result.err()
        );
        let (_, e) = result.unwrap();
        assert!(
            matches!(e, Expr::Call(_)),
            "Expected Call for '{}', got {:?}",
            input,
            e
        );
    }
}

/// Test that the function registry contains all expected functions
#[test]
fn test_all_functions_in_registry() {
    for (name, min_args, max_args, _return_type) in FUNCTION_SIGNATURES {
        let func = get_function(name);
        assert!(func.is_some(), "Function '{}' not found in registry", name);

        let func = func.unwrap();
        assert_eq!(
            func.min_args(),
            *min_args as usize,
            "Function '{}' has wrong min_args: expected {}, got {}",
            name,
            min_args,
            func.min_args()
        );

        // max_args returns Option<usize> for variadic functions
        let expected_max = if *max_args == 255 {
            None // 255 means variadic (unlimited)
        } else {
            Some(*max_args as usize)
        };
        assert_eq!(
            func.max_args(),
            expected_max,
            "Function '{}' has wrong max_args: expected {:?}, got {:?}",
            name,
            expected_max,
            func.max_args()
        );
    }
}

/// Test that function call parsing extracts the correct function name
#[test]
fn test_function_call_extracts_name() {
    let test_cases = &[
        ("time()", "time"),
        ("rate(some_metric[5m])", "rate"),
        ("ceil(some_metric)", "ceil"),
        ("clamp(metric, 0, 100)", "clamp"),
        (
            r#"label_replace(metric, "dst", "$1", "src", "(.*)")"#,
            "label_replace",
        ),
    ];

    for (input, expected_name) in test_cases {
        let (rest, e) = expr(input).unwrap();
        assert!(rest.is_empty(), "Unparsed input remaining: {:?}", rest);
        if let Expr::Call(call) = e {
            assert_eq!(
                call.name, *expected_name,
                "Function name mismatch for input: {:?}",
                input
            );
        } else {
            panic!("Expected Call for '{}'", input);
        }
    }
}

/// Test that function call parsing counts arguments correctly
#[test]
fn test_function_call_argument_count() {
    let test_cases = &[
        ("time()", 0),
        ("abs(metric)", 1),
        ("clamp(metric, 0, 100)", 3),
        (r#"label_replace(metric, "dst", "$1", "src", "(.*)")"#, 5),
    ];

    for (input, expected_count) in test_cases {
        let (_, e) = expr(input).unwrap();
        if let Expr::Call(call) = e {
            assert_eq!(
                call.args.len(),
                *expected_count,
                "Argument count mismatch for input: {:?}",
                input
            );
        } else {
            panic!("Expected Call for '{}'", input);
        }
    }
}

/// Test function call Display implementation
#[test]
fn test_function_call_display() {
    let (_, e) = expr("rate(http_requests[5m])").unwrap();
    let display = format!("{}", e);
    assert!(
        display.contains("rate"),
        "Display should contain function name"
    );
    assert!(display.contains("("), "Display should contain parentheses");
}

/// Test nested function calls parse correctly
#[test]
fn test_nested_function_calls() {
    let input = "histogram_quantile(0.9, rate(http_requests_total[5m]))";
    let result = expr(input);
    assert!(
        result.is_ok(),
        "Failed to parse nested function call: {:?}",
        result.err()
    );

    let (rest, e) = result.unwrap();
    assert!(rest.is_empty());
    if let Expr::Call(call) = e {
        assert_eq!(call.name, "histogram_quantile");
        assert_eq!(call.args.len(), 2);
    } else {
        panic!("Expected Call");
    }
}

/// Test that whitespace is handled correctly in function calls
#[test]
fn test_function_call_whitespace() {
    let test_cases = &[
        "rate( http_requests[5m] )",
        "clamp( metric , 0 , 100 )",
        "time( )",
    ];

    for input in test_cases {
        let result = expr(input);
        assert!(
            result.is_ok(),
            "Failed to parse function call with whitespace: {:?}\nError: {:?}",
            input,
            result.err()
        );
    }
}
