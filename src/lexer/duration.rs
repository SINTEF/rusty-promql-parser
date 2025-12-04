// Duration literal parser for PromQL
//
// Durations represent time spans and are used in:
// - Range selectors: metric[5m]
// - Offset modifiers: metric offset 1h
// - Subqueries: metric[30m:5m]
//
// Format: <number><unit>[<number><unit>...]
// Units (in descending size order):
//   y  - year (365 days)
//   w  - week (7 days)
//   d  - day (24 hours)
//   h  - hour
//   m  - minute
//   s  - second
//   ms - millisecond
//
// Examples: 5m, 1h30m, 2d12h, 100ms

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res, opt},
    multi::many1,
    sequence::pair,
};

/// Duration value in milliseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    /// Duration in milliseconds (can be negative for negative offsets)
    pub milliseconds: i64,
}

impl Duration {
    /// Create a new duration from milliseconds
    pub const fn from_millis(ms: i64) -> Self {
        Self { milliseconds: ms }
    }

    /// Create a new duration from seconds
    pub const fn from_secs(secs: i64) -> Self {
        Self {
            milliseconds: secs * 1000,
        }
    }

    /// Get the duration in milliseconds
    pub const fn as_millis(&self) -> i64 {
        self.milliseconds
    }

    /// Get the duration in seconds (truncated)
    pub const fn as_secs(&self) -> i64 {
        self.milliseconds / 1000
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ms = self.milliseconds;
        if ms == 0 {
            return write!(f, "0s");
        }

        let mut result = String::new();

        // Handle negative durations
        if ms < 0 {
            result.push('-');
            ms = -ms;
        }

        // Handle years
        let years = ms / 31_536_000_000;
        if years > 0 {
            result.push_str(&format!("{}y", years));
            ms %= 31_536_000_000;
        }

        // Handle weeks
        let weeks = ms / 604_800_000;
        if weeks > 0 {
            result.push_str(&format!("{}w", weeks));
            ms %= 604_800_000;
        }

        // Handle days
        let days = ms / 86_400_000;
        if days > 0 {
            result.push_str(&format!("{}d", days));
            ms %= 86_400_000;
        }

        // Handle hours
        let hours = ms / 3_600_000;
        if hours > 0 {
            result.push_str(&format!("{}h", hours));
            ms %= 3_600_000;
        }

        // Handle minutes
        let minutes = ms / 60_000;
        if minutes > 0 {
            result.push_str(&format!("{}m", minutes));
            ms %= 60_000;
        }

        // Handle seconds
        let seconds = ms / 1000;
        if seconds > 0 {
            result.push_str(&format!("{}s", seconds));
            ms %= 1000;
        }

        // Handle milliseconds
        if ms > 0 {
            result.push_str(&format!("{}ms", ms));
        }

        write!(f, "{}", result)
    }
}

/// Duration unit with its millisecond multiplier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DurationUnit {
    Millisecond, // ms - 1
    Second,      // s  - 1,000
    Minute,      // m  - 60,000
    Hour,        // h  - 3,600,000
    Day,         // d  - 86,400,000
    Week,        // w  - 604,800,000
    Year,        // y  - 31,536,000,000
}

impl DurationUnit {
    /// Get the multiplier in milliseconds
    const fn millis(&self) -> i64 {
        match self {
            DurationUnit::Millisecond => 1,
            DurationUnit::Second => 1_000,
            DurationUnit::Minute => 60_000,
            DurationUnit::Hour => 3_600_000,
            DurationUnit::Day => 86_400_000,
            DurationUnit::Week => 604_800_000,
            DurationUnit::Year => 31_536_000_000,
        }
    }
}

/// Compute the total duration in milliseconds from components, with overflow checking.
fn compute_duration_millis(components: Vec<(i64, DurationUnit)>) -> Result<Duration, ()> {
    let mut total_ms: i64 = 0;
    for (value, unit) in components {
        let component_ms = value.checked_mul(unit.millis()).ok_or(())?;
        total_ms = total_ms.checked_add(component_ms).ok_or(())?;
    }
    Ok(Duration::from_millis(total_ms))
}

/// Parse a PromQL duration literal.
///
/// Returns the Duration with total milliseconds.
/// Compound durations like "1h30m" are supported.
/// Returns an error if the duration would overflow i64.
pub fn duration(input: &str) -> IResult<&str, Duration> {
    map_res(many1(duration_component), compute_duration_millis).parse(input)
}

/// Parse a single duration component: <number><unit>
fn duration_component(input: &str) -> IResult<&str, (i64, DurationUnit)> {
    pair(map_res(digit1, |s: &str| s.parse::<i64>()), duration_unit).parse(input)
}

/// Parse a duration unit
fn duration_unit(input: &str) -> IResult<&str, DurationUnit> {
    alt((
        // "ms" must come before "m" to match correctly
        map(tag("ms"), |_| DurationUnit::Millisecond),
        map(tag("s"), |_| DurationUnit::Second),
        map(tag("m"), |_| DurationUnit::Minute),
        map(tag("h"), |_| DurationUnit::Hour),
        map(tag("d"), |_| DurationUnit::Day),
        map(tag("w"), |_| DurationUnit::Week),
        map(tag("y"), |_| DurationUnit::Year),
    ))
    .parse(input)
}

/// Parse a duration that may be preceded by a sign (+/-).
/// Used for offset modifiers which can be negative.
pub fn signed_duration(input: &str) -> IResult<&str, Duration> {
    map(
        pair(opt(alt((tag("+"), tag("-")))), duration),
        |(sign, dur)| {
            if sign == Some("-") {
                Duration::from_millis(-dur.milliseconds)
            } else {
                dur
            }
        },
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to test duration parsing
    fn assert_duration(input: &str, expected_ms: i64) {
        let result = duration(input);
        match result {
            Ok((remaining, dur)) => {
                assert!(
                    remaining.is_empty(),
                    "Parser did not consume entire input '{}', remaining: '{}'",
                    input,
                    remaining
                );
                assert_eq!(
                    dur.milliseconds, expected_ms,
                    "For input '{}', expected {}ms, got {}ms",
                    input, expected_ms, dur.milliseconds
                );
            }
            Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
        }
    }

    /// Helper to test signed duration parsing
    fn assert_signed_duration(input: &str, expected_ms: i64) {
        let result = signed_duration(input);
        match result {
            Ok((remaining, dur)) => {
                assert!(
                    remaining.is_empty(),
                    "Parser did not consume entire input '{}', remaining: '{}'",
                    input,
                    remaining
                );
                assert_eq!(
                    dur.milliseconds, expected_ms,
                    "For input '{}', expected {}ms, got {}ms",
                    input, expected_ms, dur.milliseconds
                );
            }
            Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
        }
    }

    // Simple durations
    #[test]
    fn test_milliseconds() {
        assert_duration("1ms", 1);
        assert_duration("100ms", 100);
        assert_duration("1000ms", 1000);
    }

    #[test]
    fn test_seconds() {
        assert_duration("1s", 1_000);
        assert_duration("5s", 5_000);
        assert_duration("30s", 30_000);
    }

    #[test]
    fn test_minutes() {
        assert_duration("1m", 60_000);
        assert_duration("5m", 300_000);
        assert_duration("30m", 1_800_000);
        assert_duration("123m", 7_380_000);
    }

    #[test]
    fn test_hours() {
        assert_duration("1h", 3_600_000);
        assert_duration("5h", 18_000_000);
        assert_duration("24h", 86_400_000);
    }

    #[test]
    fn test_days() {
        assert_duration("1d", 86_400_000);
        assert_duration("5d", 432_000_000);
    }

    #[test]
    fn test_weeks() {
        assert_duration("1w", 604_800_000);
        assert_duration("3w", 1_814_400_000);
        assert_duration("5w", 3_024_000_000);
    }

    #[test]
    fn test_years() {
        assert_duration("1y", 31_536_000_000);
        assert_duration("5y", 157_680_000_000);
    }

    // Compound durations
    #[test]
    fn test_compound_hour_minute() {
        assert_duration("1h30m", 5_400_000);
    }

    #[test]
    fn test_compound_minute_second() {
        assert_duration("5m30s", 330_000);
    }

    #[test]
    fn test_compound_second_millisecond() {
        assert_duration("4s180ms", 4_180);
        assert_duration("4s18ms", 4_018);
        assert_duration("1m30ms", 60_030);
    }

    #[test]
    fn test_compound_complex() {
        assert_duration("1h30m15s", 5_415_000);
        assert_duration("2d12h", 216_000_000);
        assert_duration("5m10s", 310_000);
    }

    // Signed durations
    #[test]
    fn test_signed_positive() {
        assert_signed_duration("+5m", 300_000);
        assert_signed_duration("+1h30m", 5_400_000);
    }

    #[test]
    fn test_signed_negative() {
        assert_signed_duration("-5m", -300_000);
        assert_signed_duration("-7m", -420_000);
        assert_signed_duration("-1h30m", -5_400_000);
    }

    #[test]
    fn test_signed_no_sign() {
        // Without sign, should parse same as unsigned
        assert_signed_duration("5m", 300_000);
    }

    // Display formatting
    #[test]
    fn test_duration_display() {
        assert_eq!(Duration::from_millis(0).to_string(), "0s");
        assert_eq!(Duration::from_millis(1).to_string(), "1ms");
        assert_eq!(Duration::from_millis(1000).to_string(), "1s");
        assert_eq!(Duration::from_millis(60_000).to_string(), "1m");
        assert_eq!(Duration::from_millis(3_600_000).to_string(), "1h");
        assert_eq!(Duration::from_millis(5_400_000).to_string(), "1h30m");
        assert_eq!(Duration::from_millis(86_400_000).to_string(), "1d");
        assert_eq!(Duration::from_millis(604_800_000).to_string(), "1w");
        assert_eq!(Duration::from_millis(31_536_000_000).to_string(), "1y");
    }

    // Edge cases
    #[test]
    fn test_partial_parse() {
        // Duration followed by other content
        let (remaining, dur) = duration("5m30s offset").unwrap();
        assert_eq!(dur.milliseconds, 330_000);
        assert_eq!(remaining, " offset");
    }

    #[test]
    fn test_invalid_unit() {
        // Invalid unit should fail
        assert!(duration("5x").is_err());
        assert!(duration("5").is_err()); // Number without unit
    }

    #[test]
    fn test_fail_found_with_fuzzing() {
        assert!(duration("5555555555555555555m").is_err());
    }
}
