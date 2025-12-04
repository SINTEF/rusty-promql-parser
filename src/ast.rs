//! AST type definitions for PromQL expressions
//!
//! This module defines the Abstract Syntax Tree types for PromQL.
//! The main entry point is the `Expr` enum which represents any valid PromQL expression.

use std::fmt;

use crate::lexer::duration::Duration;
use crate::parser::aggregation::Grouping;
use crate::parser::selector::{AtModifier, MatrixSelector, VectorSelector};

/// Root expression type for PromQL AST
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal: `42`, `3.14`, `0x1F`, `1e-10`, `Inf`, `NaN`
    Number(f64),

    /// String literal: `"hello"`, `'world'`, `` `raw` ``
    String(String),

    /// Instant vector selector: `http_requests{job="api"}`
    VectorSelector(VectorSelector),

    /// Range vector selector: `http_requests{job="api"}[5m]`
    MatrixSelector(MatrixSelector),

    /// Function call: `rate(http_requests[5m])`
    Call(Call),

    /// Aggregation: `sum by (job) (http_requests)`
    Aggregation(Box<Aggregation>),

    /// Binary operation: `foo + bar`, `foo / on(job) bar`
    Binary(Box<BinaryExpr>),

    /// Unary operation: `-foo`, `+bar`
    Unary(Box<UnaryExpr>),

    /// Parenthesized: `(foo + bar)`
    Paren(Box<Expr>),

    /// Subquery: `rate(http_requests[5m])[30m:1m]`
    Subquery(Box<SubqueryExpr>),
}

impl Expr {
    /// Check if this is a scalar expression (number literal)
    pub fn is_scalar(&self) -> bool {
        matches!(self, Expr::Number(_))
    }

    /// Check if this is a string literal
    pub fn is_string(&self) -> bool {
        matches!(self, Expr::String(_))
    }

    /// Check if this expression produces an instant vector
    pub fn is_instant_vector(&self) -> bool {
        matches!(
            self,
            Expr::VectorSelector(_)
                | Expr::Call(_)
                | Expr::Aggregation(_)
                | Expr::Binary(_)
                | Expr::Unary(_)
        ) || matches!(self, Expr::Paren(e) if e.is_instant_vector())
    }

    /// Check if this expression produces a range vector
    pub fn is_range_vector(&self) -> bool {
        matches!(self, Expr::MatrixSelector(_) | Expr::Subquery(_))
    }

    /// Unwrap parentheses to get the inner expression
    pub fn unwrap_parens(&self) -> &Expr {
        match self {
            Expr::Paren(inner) => inner.unwrap_parens(),
            other => other,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) => {
                if n.is_nan() {
                    write!(f, "NaN")
                } else if n.is_infinite() {
                    if *n > 0.0 {
                        write!(f, "Inf")
                    } else {
                        write!(f, "-Inf")
                    }
                } else {
                    write!(f, "{}", n)
                }
            }
            Expr::String(s) => write!(f, "\"{}\"", s.escape_default()),
            Expr::VectorSelector(v) => write!(f, "{}", v),
            Expr::MatrixSelector(m) => write!(f, "{}", m),
            Expr::Call(c) => write!(f, "{}", c),
            Expr::Aggregation(a) => write!(f, "{}", a),
            Expr::Binary(b) => write!(f, "{}", b),
            Expr::Unary(u) => write!(f, "{}", u),
            Expr::Paren(e) => write!(f, "({})", e),
            Expr::Subquery(s) => write!(f, "{}", s),
        }
    }
}

/// Function call expression
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    /// Function name
    pub name: String,
    /// Function arguments
    pub args: Vec<Expr>,
}

impl Call {
    /// Create a new function call
    pub fn new(name: impl Into<String>, args: Vec<Expr>) -> Self {
        Self {
            name: name.into(),
            args,
        }
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ")")
    }
}

/// Aggregation expression
#[derive(Debug, Clone, PartialEq)]
pub struct Aggregation {
    /// The aggregation operator name
    pub op: String,
    /// The expression to aggregate
    pub expr: Expr,
    /// Parameter for parametric aggregations (topk, quantile, etc.)
    pub param: Option<Expr>,
    /// Optional grouping clause (by/without)
    pub grouping: Option<Grouping>,
}

impl Aggregation {
    /// Create a new aggregation
    pub fn new(op: impl Into<String>, expr: Expr) -> Self {
        Self {
            op: op.into(),
            expr,
            param: None,
            grouping: None,
        }
    }

    /// Create a new aggregation with a parameter
    pub fn with_param(op: impl Into<String>, param: Expr, expr: Expr) -> Self {
        Self {
            op: op.into(),
            expr,
            param: Some(param),
            grouping: None,
        }
    }

    /// Set the grouping clause
    pub fn with_grouping(mut self, grouping: Grouping) -> Self {
        self.grouping = Some(grouping);
        self
    }
}

impl fmt::Display for Aggregation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.op)?;
        if let Some(ref grouping) = self.grouping {
            write!(f, " {} ", grouping)?;
        }
        write!(f, "(")?;
        if let Some(ref param) = self.param {
            write!(f, "{}, ", param)?;
        }
        write!(f, "{})", self.expr)
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,   // +
    Sub,   // -
    Mul,   // *
    Div,   // /
    Mod,   // %
    Pow,   // ^
    Atan2, // atan2

    // Comparison
    Eq, // ==
    Ne, // !=
    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=

    // Set operations
    And,    // and
    Or,     // or
    Unless, // unless
}

impl BinaryOp {
    /// Get the operator as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Pow => "^",
            BinaryOp::Atan2 => "atan2",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
            BinaryOp::Unless => "unless",
        }
    }

    /// Get the precedence of this operator (higher = binds tighter)
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,                     // Lowest
            BinaryOp::And | BinaryOp::Unless => 2, // Set intersection/difference
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => 3, // Comparison
            BinaryOp::Add | BinaryOp::Sub => 4,    // Addition/subtraction
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod | BinaryOp::Atan2 => 5, // Multiplication/division
            BinaryOp::Pow => 6,                                                   // Highest
        }
    }

    /// Check if this operator is right-associative
    pub fn is_right_associative(&self) -> bool {
        matches!(self, BinaryOp::Pow)
    }

    /// Check if this is a comparison operator
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
        )
    }

    /// Check if this is a set operator
    pub fn is_set_operator(&self) -> bool {
        matches!(self, BinaryOp::And | BinaryOp::Or | BinaryOp::Unless)
    }

    /// Check if this is an arithmetic operator
    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            BinaryOp::Add
                | BinaryOp::Sub
                | BinaryOp::Mul
                | BinaryOp::Div
                | BinaryOp::Mod
                | BinaryOp::Pow
                | BinaryOp::Atan2
        )
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Vector matching for binary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorMatchingOp {
    On,       // on (label1, label2)
    Ignoring, // ignoring (label1, label2)
}

impl fmt::Display for VectorMatchingOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VectorMatchingOp::On => write!(f, "on"),
            VectorMatchingOp::Ignoring => write!(f, "ignoring"),
        }
    }
}

/// Group modifier side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupSide {
    Left,  // group_left
    Right, // group_right
}

impl fmt::Display for GroupSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupSide::Left => write!(f, "group_left"),
            GroupSide::Right => write!(f, "group_right"),
        }
    }
}

/// Group modifier for many-to-one/one-to-many matching
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupModifier {
    /// Which side to group (left or right)
    pub side: GroupSide,
    /// Additional labels to include from the "one" side
    pub labels: Vec<String>,
}

impl fmt::Display for GroupModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.side)?;
        if !self.labels.is_empty() {
            write!(f, " (")?;
            for (i, label) in self.labels.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", label)?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

/// Vector matching specification for binary operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorMatching {
    /// The matching operation (on or ignoring)
    pub op: VectorMatchingOp,
    /// Labels to match on or ignore
    pub labels: Vec<String>,
    /// Optional group modifier
    pub group: Option<GroupModifier>,
}

impl fmt::Display for VectorMatching {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (", self.op)?;
        for (i, label) in self.labels.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", label)?;
        }
        write!(f, ")")?;
        if let Some(ref group) = self.group {
            write!(f, " {}", group)?;
        }
        Ok(())
    }
}

/// Modifier for binary operations
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BinaryModifier {
    /// Whether to return bool (0/1) instead of filtering for comparisons
    pub return_bool: bool,
    /// Vector matching specification
    pub matching: Option<VectorMatching>,
}

impl BinaryModifier {
    /// Create a modifier with just the bool flag
    pub fn with_bool() -> Self {
        Self {
            return_bool: true,
            matching: None,
        }
    }

    /// Create a modifier with vector matching
    pub fn with_matching(matching: VectorMatching) -> Self {
        Self {
            return_bool: false,
            matching: Some(matching),
        }
    }

    /// Check if this modifier has any settings
    pub fn is_empty(&self) -> bool {
        !self.return_bool && self.matching.is_none()
    }
}

impl fmt::Display for BinaryModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.return_bool {
            write!(f, "bool")?;
            if self.matching.is_some() {
                write!(f, " ")?;
            }
        }
        if let Some(ref matching) = self.matching {
            write!(f, "{}", matching)?;
        }
        Ok(())
    }
}

/// Binary expression
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    /// The binary operator
    pub op: BinaryOp,
    /// Left-hand side expression
    pub lhs: Expr,
    /// Right-hand side expression
    pub rhs: Expr,
    /// Optional modifier (bool, on, ignoring, group_left, group_right)
    pub modifier: Option<BinaryModifier>,
}

impl BinaryExpr {
    /// Create a new binary expression
    pub fn new(op: BinaryOp, lhs: Expr, rhs: Expr) -> Self {
        Self {
            op,
            lhs,
            rhs,
            modifier: None,
        }
    }

    /// Create a new binary expression with a modifier
    pub fn with_modifier(op: BinaryOp, lhs: Expr, rhs: Expr, modifier: BinaryModifier) -> Self {
        Self {
            op,
            lhs,
            rhs,
            modifier: Some(modifier),
        }
    }
}

impl fmt::Display for BinaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.lhs, self.op)?;
        if let Some(ref modifier) = self.modifier
            && !modifier.is_empty()
        {
            write!(f, " {}", modifier)?;
        }
        write!(f, " {}", self.rhs)
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Unary plus (no-op)
    Plus,
    /// Unary minus (negation)
    Minus,
}

impl UnaryOp {
    /// Get the operator as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::Plus => "+",
            UnaryOp::Minus => "-",
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Unary expression
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    /// The unary operator
    pub op: UnaryOp,
    /// The operand expression
    pub expr: Expr,
}

impl UnaryExpr {
    /// Create a new unary expression
    pub fn new(op: UnaryOp, expr: Expr) -> Self {
        Self { op, expr }
    }
}

impl fmt::Display for UnaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.op, self.expr)
    }
}

/// Subquery expression
#[derive(Debug, Clone, PartialEq)]
pub struct SubqueryExpr {
    /// The inner expression to evaluate as a subquery
    pub expr: Expr,
    /// The time range of the subquery
    pub range: Duration,
    /// Optional step/resolution (if None, uses default evaluation interval)
    pub step: Option<Duration>,
    /// Offset modifier
    pub offset: Option<Duration>,
    /// @ modifier for timestamp pinning
    pub at: Option<AtModifier>,
}

impl SubqueryExpr {
    /// Create a new subquery expression
    pub fn new(expr: Expr, range: Duration) -> Self {
        Self {
            expr,
            range,
            step: None,
            offset: None,
            at: None,
        }
    }

    /// Create a new subquery expression with step
    pub fn with_step(expr: Expr, range: Duration, step: Duration) -> Self {
        Self {
            expr,
            range,
            step: Some(step),
            offset: None,
            at: None,
        }
    }
}

impl fmt::Display for SubqueryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}:", self.expr, self.range)?;
        if let Some(ref step) = self.step {
            write!(f, "{}", step)?;
        }
        write!(f, "]")?;
        if let Some(ref at) = self.at {
            write!(f, " {}", at)?;
        }
        if let Some(ref offset) = self.offset {
            write!(f, " offset {}", offset)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_op_precedence() {
        // or < and/unless < comparison < +/- < */% < ^
        assert!(BinaryOp::Or.precedence() < BinaryOp::And.precedence());
        assert!(BinaryOp::And.precedence() < BinaryOp::Eq.precedence());
        assert!(BinaryOp::Eq.precedence() < BinaryOp::Add.precedence());
        assert!(BinaryOp::Add.precedence() < BinaryOp::Mul.precedence());
        assert!(BinaryOp::Mul.precedence() < BinaryOp::Pow.precedence());
    }

    #[test]
    fn test_binary_op_associativity() {
        assert!(!BinaryOp::Add.is_right_associative());
        assert!(!BinaryOp::Mul.is_right_associative());
        assert!(BinaryOp::Pow.is_right_associative());
    }

    #[test]
    fn test_binary_op_categories() {
        assert!(BinaryOp::Add.is_arithmetic());
        assert!(BinaryOp::Eq.is_comparison());
        assert!(BinaryOp::And.is_set_operator());
    }

    #[test]
    fn test_expr_display_number() {
        assert_eq!(Expr::Number(42.0).to_string(), "42");
        assert_eq!(Expr::Number(3.5).to_string(), "3.5");
        assert_eq!(Expr::Number(f64::INFINITY).to_string(), "Inf");
        assert_eq!(Expr::Number(f64::NEG_INFINITY).to_string(), "-Inf");
        assert_eq!(Expr::Number(f64::NAN).to_string(), "NaN");
    }

    #[test]
    fn test_expr_display_string() {
        assert_eq!(Expr::String("hello".to_string()).to_string(), "\"hello\"");
    }

    #[test]
    fn test_unary_expr_display() {
        let expr = UnaryExpr::new(UnaryOp::Minus, Expr::Number(42.0));
        assert_eq!(expr.to_string(), "-42");

        let expr = UnaryExpr::new(UnaryOp::Plus, Expr::Number(42.0));
        assert_eq!(expr.to_string(), "+42");
    }

    #[test]
    fn test_binary_expr_display() {
        let expr = BinaryExpr::new(BinaryOp::Add, Expr::Number(1.0), Expr::Number(2.0));
        assert_eq!(expr.to_string(), "1 + 2");
    }

    #[test]
    fn test_call_display() {
        let call = Call::new(
            "rate",
            vec![Expr::VectorSelector(VectorSelector::new("http_requests"))],
        );
        assert_eq!(call.to_string(), "rate(http_requests)");
    }

    #[test]
    fn test_aggregation_display() {
        let agg = Aggregation::new("sum", Expr::VectorSelector(VectorSelector::new("metric")));
        assert_eq!(agg.to_string(), "sum(metric)");
    }

    #[test]
    fn test_expr_is_scalar() {
        assert!(Expr::Number(42.0).is_scalar());
        assert!(!Expr::String("test".to_string()).is_scalar());
    }

    #[test]
    fn test_expr_unwrap_parens() {
        let inner = Expr::Number(42.0);
        let paren = Expr::Paren(Box::new(inner.clone()));
        let double_paren = Expr::Paren(Box::new(paren.clone()));

        assert_eq!(*double_paren.unwrap_parens(), inner);
    }
}
