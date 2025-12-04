// Lexer test module
// Re-exports test cases extracted from reference implementations

pub mod duration_tests;
pub mod identifier_tests;
pub mod number_tests;
pub mod string_tests;

// Re-export all test data for convenience
pub use duration_tests::*;
pub use identifier_tests::*;
pub use number_tests::*;
pub use string_tests::*;
