//! PromQL expression parser.
//!
//! This module contains the parser for PromQL expressions. The main entry point
//! is the [`expr()`] function which parses any valid PromQL expression.
//!
//! # Submodules
//!
//! - [`aggregation`] - Aggregation grouping clauses (`by`, `without`)
//! - [`binary`] - Binary operators and modifiers
//! - [`mod@expr`] - Main expression parser
//! - [`function`] - Built-in function definitions
//! - [`selector`] - Vector and matrix selectors
//! - [`subquery`] - Subquery expression parsing
//! - [`unary`] - Unary operators
//!
//! # Example
//!
//! ```rust
//! use rusty_promql_parser::parser::expr::expr;
//!
//! let (rest, ast) = expr("sum(rate(http_requests[5m])) by (job)").unwrap();
//! assert!(rest.is_empty());
//! ```

pub mod aggregation;
pub mod binary;
pub mod expr;
pub mod function;
pub mod selector;
pub mod subquery;
pub mod unary;

// Re-export the main expression parser
pub use expr::expr;
