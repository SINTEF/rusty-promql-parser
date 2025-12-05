//! Lexer module for PromQL.
//!
//! This module provides low-level parsers for individual tokens and lexemes
//! used in PromQL expressions. These parsers are building blocks used by the
//! higher-level expression parser.
//!
//! # Submodules
//!
//! - [`mod@duration`] - Duration literals (`5m`, `1h30m`, `2d`)
//! - [`identifier`] - Metric names, label names, and keywords
//! - [`mod@number`] - Numeric literals (integers, floats, hex, scientific notation)
//! - [`string`] - String literals (double-quoted, single-quoted, backtick)
//! - [`whitespace`] - Whitespace and comment handling
//!
//! # Example
//!
//! ```rust
//! use rusty_promql_parser::lexer::number::number;
//!
//! let (rest, value) = number("42.5").unwrap();
//! assert_eq!(value, 42.5);
//! ```

pub mod duration;
pub mod identifier;
pub mod number;
pub mod string;
pub mod whitespace;

pub use duration::*;
pub use number::*;
pub use string::*;
pub use whitespace::*;
