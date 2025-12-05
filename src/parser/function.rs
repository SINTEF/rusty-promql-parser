//! Built-in PromQL function definitions.
//!
//! This module defines the signatures of all built-in PromQL functions.
//! It can be used for validation and documentation purposes.
//!
//! # Function Categories
//!
//! - **Math functions**: `abs`, `ceil`, `floor`, `exp`, `sqrt`, `ln`, `log2`, `log10`
//! - **Trigonometric**: `acos`, `asin`, `atan`, `cos`, `sin`, `tan` (and hyperbolic variants)
//! - **Rounding/clamping**: `round`, `clamp`, `clamp_min`, `clamp_max`
//! - **Sorting**: `sort`, `sort_desc`, `sort_by_label`
//! - **Rate functions**: `rate`, `irate`, `increase`, `delta`, `idelta`, `deriv`
//! - **Aggregation over time**: `avg_over_time`, `sum_over_time`, `min_over_time`, etc.
//! - **Time functions**: `time`, `timestamp`, `hour`, `minute`, `month`, `year`
//! - **Label functions**: `label_replace`, `label_join`
//! - **Histogram functions**: `histogram_quantile`, `histogram_avg`, `histogram_count`
//!
//! # Example
//!
//! ```rust
//! use rusty_promql_parser::parser::function::{get_function, is_function};
//!
//! assert!(is_function("rate"));
//! assert!(!is_function("unknown_func"));
//!
//! let func = get_function("rate").unwrap();
//! assert_eq!(func.name, "rate");
//! assert_eq!(func.min_args(), 1);
//! ```

/// Value types for function arguments and return values.
///
/// PromQL has four fundamental value types that functions operate on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    /// Scalar value (single number).
    Scalar,
    /// Instant vector (set of time series with single sample each).
    Vector,
    /// Range vector (set of time series with samples over time range).
    Matrix,
    /// String value.
    String,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Scalar => write!(f, "scalar"),
            ValueType::Vector => write!(f, "instant vector"),
            ValueType::Matrix => write!(f, "range vector"),
            ValueType::String => write!(f, "string"),
        }
    }
}

/// Variadic argument specification.
///
/// Determines how many arguments a function accepts beyond the required ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variadic {
    /// Fixed number of arguments (only those specified in `arg_types`).
    None,
    /// Last argument type can repeat indefinitely.
    Repeat,
    /// Optional trailing arguments (number indicates how many of `arg_types` are optional).
    Optional(u8),
}

/// Function signature definition.
///
/// Describes the name, argument types, return type, and variadic behavior
/// of a built-in PromQL function.
#[derive(Debug, Clone)]
pub struct Function {
    /// Function name.
    pub name: &'static str,
    /// Argument types (in order).
    pub arg_types: &'static [ValueType],
    /// Variadic specification.
    pub variadic: Variadic,
    /// Return type.
    pub return_type: ValueType,
    /// Whether this is an experimental function.
    pub experimental: bool,
}

impl Function {
    /// Get the minimum number of arguments.
    ///
    /// - For `Variadic::Repeat`: `arg_types.len() - 1`
    /// - For `Variadic::Optional(n)`: `arg_types.len() - n`
    /// - For `Variadic::None`: `arg_types.len()`
    pub fn min_args(&self) -> usize {
        match self.variadic {
            Variadic::None => self.arg_types.len(),
            Variadic::Repeat => self.arg_types.len().saturating_sub(1),
            Variadic::Optional(n) => self.arg_types.len().saturating_sub(n as usize),
        }
    }

    /// Get the maximum number of arguments.
    ///
    /// Returns `None` for variadic functions that accept unlimited arguments.
    pub fn max_args(&self) -> Option<usize> {
        match self.variadic {
            Variadic::None => Some(self.arg_types.len()),
            Variadic::Repeat => None, // Unlimited
            Variadic::Optional(_) => Some(self.arg_types.len()),
        }
    }
}

/// All built-in PromQL functions.
///
/// This static array contains the definitions of all standard PromQL functions
/// as defined in the Prometheus documentation.
pub static FUNCTIONS: &[Function] = &[
    // Math functions
    Function {
        name: "abs",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "ceil",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "floor",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "exp",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "sqrt",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "ln",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "log2",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "log10",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "sgn",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "deg",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "rad",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    // Trigonometric functions
    Function {
        name: "acos",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "acosh",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "asin",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "asinh",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "atan",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "atanh",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "cos",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "cosh",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "sin",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "sinh",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "tan",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "tanh",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    // Rounding/clamping functions
    Function {
        name: "round",
        arg_types: &[ValueType::Vector, ValueType::Scalar],
        variadic: Variadic::Optional(1),
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "clamp",
        arg_types: &[ValueType::Vector, ValueType::Scalar, ValueType::Scalar],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "clamp_min",
        arg_types: &[ValueType::Vector, ValueType::Scalar],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "clamp_max",
        arg_types: &[ValueType::Vector, ValueType::Scalar],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    // Sorting functions
    Function {
        name: "sort",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "sort_desc",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "sort_by_label",
        arg_types: &[ValueType::Vector, ValueType::String],
        variadic: Variadic::Repeat,
        return_type: ValueType::Vector,
        experimental: true,
    },
    Function {
        name: "sort_by_label_desc",
        arg_types: &[ValueType::Vector, ValueType::String],
        variadic: Variadic::Repeat,
        return_type: ValueType::Vector,
        experimental: true,
    },
    // Rate/counter functions (range vector -> instant vector)
    Function {
        name: "rate",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "irate",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "increase",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "delta",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "idelta",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "deriv",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "changes",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "resets",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    // Over-time aggregation functions (range vector -> instant vector)
    Function {
        name: "avg_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "sum_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "count_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "min_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "max_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "stddev_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "stdvar_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "last_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "first_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: true,
    },
    Function {
        name: "present_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "absent_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "quantile_over_time",
        arg_types: &[ValueType::Scalar, ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "mad_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: true,
    },
    // Timestamp functions
    Function {
        name: "ts_of_first_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: true,
    },
    Function {
        name: "ts_of_max_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: true,
    },
    Function {
        name: "ts_of_min_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: true,
    },
    Function {
        name: "ts_of_last_over_time",
        arg_types: &[ValueType::Matrix],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: true,
    },
    // Time functions
    Function {
        name: "time",
        arg_types: &[],
        variadic: Variadic::None,
        return_type: ValueType::Scalar,
        experimental: false,
    },
    Function {
        name: "timestamp",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "hour",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::Optional(1),
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "minute",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::Optional(1),
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "month",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::Optional(1),
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "year",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::Optional(1),
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "day_of_week",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::Optional(1),
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "day_of_month",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::Optional(1),
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "day_of_year",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::Optional(1),
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "days_in_month",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::Optional(1),
        return_type: ValueType::Vector,
        experimental: false,
    },
    // Label functions
    Function {
        name: "label_replace",
        arg_types: &[
            ValueType::Vector,
            ValueType::String,
            ValueType::String,
            ValueType::String,
            ValueType::String,
        ],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "label_join",
        arg_types: &[
            ValueType::Vector,
            ValueType::String,
            ValueType::String,
            ValueType::String,
        ],
        variadic: Variadic::Repeat,
        return_type: ValueType::Vector,
        experimental: false,
    },
    // Other functions
    Function {
        name: "absent",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "scalar",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Scalar,
        experimental: false,
    },
    Function {
        name: "vector",
        arg_types: &[ValueType::Scalar],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "predict_linear",
        arg_types: &[ValueType::Matrix, ValueType::Scalar],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "pi",
        arg_types: &[],
        variadic: Variadic::None,
        return_type: ValueType::Scalar,
        experimental: false,
    },
    // Histogram functions
    Function {
        name: "histogram_quantile",
        arg_types: &[ValueType::Scalar, ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "histogram_avg",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "histogram_count",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "histogram_sum",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "histogram_stddev",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "histogram_stdvar",
        arg_types: &[ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "histogram_fraction",
        arg_types: &[ValueType::Scalar, ValueType::Scalar, ValueType::Vector],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: false,
    },
    Function {
        name: "double_exponential_smoothing",
        arg_types: &[ValueType::Matrix, ValueType::Scalar, ValueType::Scalar],
        variadic: Variadic::None,
        return_type: ValueType::Vector,
        experimental: true,
    },
    // Info function
    Function {
        name: "info",
        arg_types: &[ValueType::Vector, ValueType::Vector],
        variadic: Variadic::Optional(1),
        return_type: ValueType::Vector,
        experimental: true,
    },
];

/// Look up a function by name.
///
/// Returns `None` if the function is not a known built-in.
pub fn get_function(name: &str) -> Option<&'static Function> {
    FUNCTIONS.iter().find(|f| f.name == name)
}

/// Check if a name is a known built-in function.
pub fn is_function(name: &str) -> bool {
    get_function(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_function() {
        assert!(get_function("rate").is_some());
        assert!(get_function("abs").is_some());
        assert!(get_function("nonexistent").is_none());
    }

    #[test]
    fn test_function_min_max_args() {
        let rate = get_function("rate").unwrap();
        assert_eq!(rate.min_args(), 1);
        assert_eq!(rate.max_args(), Some(1));

        let round = get_function("round").unwrap();
        assert_eq!(round.min_args(), 1);
        assert_eq!(round.max_args(), Some(2));

        let label_join = get_function("label_join").unwrap();
        assert_eq!(label_join.min_args(), 3);
        assert_eq!(label_join.max_args(), None);

        let time = get_function("time").unwrap();
        assert_eq!(time.min_args(), 0);
        assert_eq!(time.max_args(), Some(0));
    }

    #[test]
    fn test_all_functions_defined() {
        // Check that we have the main functions
        let expected_functions = [
            "abs",
            "ceil",
            "floor",
            "rate",
            "irate",
            "increase",
            "sum_over_time",
            "avg_over_time",
            "time",
            "timestamp",
            "label_replace",
            "label_join",
            "histogram_quantile",
            "clamp",
            "round",
            "sort",
            "sort_desc",
        ];

        for name in expected_functions {
            assert!(get_function(name).is_some(), "Missing function: {}", name);
        }
    }
}
