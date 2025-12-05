//! # Rusty PromQL Parser
//!
//! A Rust parser for the Prometheus Query Language (PromQL) using the
//! [nom](https://github.com/rust-bakery/nom) parser combinator library.
//!
//! This crate provides a complete parser for PromQL expressions, producing an
//! Abstract Syntax Tree (AST) that can be used for analysis, transformation,
//! or evaluation.
//!
//! ## Quick Start
//!
//! The main entry point is the [`expr()`] function, which parses a PromQL expression
//! and returns the remaining input along with the parsed AST:
//!
//! ```rust
//! use rusty_promql_parser::expr;
//!
//! let input = r#"http_requests_total{job="api"}"#;
//! let (rest, ast) = expr(input).expect("failed to parse");
//! assert!(rest.is_empty());
//! println!("{:#?}", ast);
//! ```
//!
//! ## Examples
//!
//! ### Parsing a metric with label filtering
//!
//! ```rust
//! use rusty_promql_parser::expr;
//!
//! let input = r#"go_gc_duration_seconds{instance="localhost:9090", job="alertmanager"}"#;
//! let (rest, ast) = expr(input).expect("failed to parse");
//! assert!(rest.is_empty());
//! ```
//!
//! ### Parsing aggregation operators
//!
//! ```rust
//! use rusty_promql_parser::expr;
//!
//! let input = r#"sum by (app, proc) (
//!   instance_memory_limit_bytes - instance_memory_usage_bytes
//! ) / 1024 / 1024"#;
//! let (rest, ast) = expr(input).expect("failed to parse");
//! assert!(rest.is_empty());
//! ```
//!
//! ### Parsing rate queries
//!
//! ```rust
//! use rusty_promql_parser::expr;
//!
//! let input = "rate(http_requests_total[5m])";
//! let (rest, ast) = expr(input).expect("failed to parse");
//! assert!(rest.is_empty());
//! ```
//!
//! ## AST Types
//!
//! The parser produces an [`Expr`] enum which can be one of:
//!
//! - [`Expr::Number`] - Numeric literals (`42`, `3.14`, `Inf`, `NaN`)
//! - [`Expr::String`] - String literals (`"hello"`, `'world'`)
//! - [`Expr::VectorSelector`] - Instant vector selectors (`metric{label="value"}`)
//! - [`Expr::MatrixSelector`] - Range vector selectors (`metric[5m]`)
//! - [`Expr::Call`] - Function calls (`rate(...)`, `histogram_quantile(...)`)
//! - [`Expr::Aggregation`] - Aggregation expressions (`sum by (job) (...)`)
//! - [`Expr::Binary`] - Binary operations (`a + b`, `foo and bar`)
//! - [`Expr::Unary`] - Unary operations (`-metric`)
//! - [`Expr::Paren`] - Parenthesized expressions (`(a + b)`)
//! - [`Expr::Subquery`] - Subqueries (`metric[5m:1m]`)
//!
//! ## Modules
//!
//! - [`ast`] - Abstract Syntax Tree type definitions
//! - [`lexer`] - Low-level token parsers (numbers, strings, durations, identifiers)
//! - [`parser`] - Expression and statement parsers
//!
//! ## Display
//!
//! All AST types implement [`std::fmt::Display`], allowing you to convert parsed
//! expressions back to PromQL strings:
//!
//! ```rust
//! use rusty_promql_parser::expr;
//!
//! let (_, ast) = expr("1 + 2 * 3").unwrap();
//! assert_eq!(ast.to_string(), "1 + 2 * 3");
//! ```

pub mod ast;
pub mod lexer;
pub mod parser;

// Re-export commonly used types and parsers
pub use ast::{
    Aggregation, BinaryExpr, BinaryModifier, BinaryOp, Call, Expr, GroupModifier, GroupSide,
    SubqueryExpr, UnaryExpr, UnaryOp, VectorMatching, VectorMatchingOp,
};
pub use lexer::number;
pub use parser::aggregation::{Grouping, GroupingAction};
pub use parser::expr;
pub use parser::selector::{LabelMatchOp, LabelMatcher, MatrixSelector, VectorSelector};
