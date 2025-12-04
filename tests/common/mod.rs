// Test helper macros and utilities for PromQL parser tests
// These macros provide consistent patterns for testing parse results

/// Assert that input parses successfully
#[macro_export]
macro_rules! assert_parses {
    ($input:expr) => {{
        let result = $crate::parse($input);
        assert!(
            result.is_ok(),
            "Expected '{}' to parse successfully, got error: {:?}",
            $input,
            result.err()
        );
        result.unwrap()
    }};
}

/// Assert that input fails to parse
#[macro_export]
macro_rules! assert_parse_error {
    ($input:expr) => {{
        let result = $crate::parse($input);
        assert!(
            result.is_err(),
            "Expected '{}' to fail parsing, but got: {:?}",
            $input,
            result.ok()
        );
    }};
    ($input:expr, $error_contains:expr) => {{
        let result = $crate::parse($input);
        assert!(
            result.is_err(),
            "Expected '{}' to fail parsing, but got: {:?}",
            $input,
            result.ok()
        );
        let err = result.err().unwrap();
        let err_str = format!("{:?}", err);
        assert!(
            err_str.contains($error_contains),
            "Expected error to contain '{}', got: {}",
            $error_contains,
            err_str
        );
    }};
}

/// Assert parse-print roundtrip produces equivalent result
#[macro_export]
macro_rules! assert_roundtrip {
    ($input:expr) => {{
        let expr = assert_parses!($input);
        let printed = expr.to_string();
        let reparsed = $crate::parse(&printed);
        assert!(
            reparsed.is_ok(),
            "Roundtrip failed: '{}' -> '{}' failed to parse: {:?}",
            $input,
            printed,
            reparsed.err()
        );
    }};
}

/// Test case structure for parameterized tests
#[derive(Debug, Clone)]
pub struct TestCase {
    pub input: &'static str,
    pub should_fail: bool,
    pub error_contains: Option<&'static str>,
    pub description: Option<&'static str>,
}

impl TestCase {
    pub const fn valid(input: &'static str) -> Self {
        Self {
            input,
            should_fail: false,
            error_contains: None,
            description: None,
        }
    }

    pub const fn invalid(input: &'static str) -> Self {
        Self {
            input,
            should_fail: true,
            error_contains: None,
            description: None,
        }
    }

    pub const fn invalid_with_error(input: &'static str, error: &'static str) -> Self {
        Self {
            input,
            should_fail: true,
            error_contains: Some(error),
            description: None,
        }
    }

    pub const fn with_description(mut self, desc: &'static str) -> Self {
        self.description = Some(desc);
        self
    }
}
