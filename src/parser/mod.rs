//! PromQL expression parser
//!
//! This module contains the parser for PromQL expressions.

pub mod aggregation;
pub mod binary;
pub mod expr;
pub mod function;
pub mod selector;
pub mod subquery;
pub mod unary;

// Re-export the main expression parser
pub use expr::expr;
