// Integration test cases for complete PromQL expressions
//
// These test cases are extracted from:
// - references/prometheus/promql/parser/parse_test.go
// - Real-world PromQL queries
//
// Integration tests verify that complex expressions combining
// multiple parser features work correctly together.

/// Complex real-world queries
pub const REAL_WORLD_QUERIES: &[&str] = &[
    // Basic request rate
    "rate(http_requests_total[5m])",
    // Request rate by service
    r#"sum(rate(http_requests_total[5m])) by (service)"#,
    // Error rate percentage
    r#"sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) * 100"#,
    // P99 latency from histogram
    r#"histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))"#,
    // P99 latency by service
    r#"histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le, service))"#,
    // Memory usage percentage
    r#"(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100"#,
    // CPU usage
    r#"100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)"#,
    // Disk space usage
    r#"(node_filesystem_size_bytes - node_filesystem_free_bytes) / node_filesystem_size_bytes * 100"#,
    // Top 10 by memory
    r#"topk(10, container_memory_usage_bytes)"#,
    // Increase over period
    "increase(http_requests_total[1h])",
    // Rate with complex selector
    r#"rate(http_requests_total{job="api-server",method="POST",status!~"5.."}[5m])"#,
    // Aggregation with grouping
    r#"sum without (instance) (rate(http_requests_total[5m]))"#,
    // Multiple aggregations
    r#"max(sum by (job) (rate(http_requests_total[5m])))"#,
    // Boolean filtering
    r#"http_requests_total > 100"#,
    // Boolean filtering with bool modifier
    r#"http_requests_total > bool 100"#,
    // Vector matching
    r#"http_requests_total / on(instance) http_requests_errors"#,
    // Group modifiers
    r#"http_requests_total / on(instance) group_left(job) http_requests_errors"#,
    // Absent check
    r#"absent(up{job="prometheus"})"#,
    // Label replace
    r#"label_replace(up, "host", "$1", "instance", "(.*):.*")"#,
    // Subquery
    "avg_over_time(rate(http_requests_total[5m])[30m:1m])",
    // Offset modifier
    "rate(http_requests_total[5m] offset 1h)",
    // @ modifier
    "http_requests_total @ 1609459200",
    // Negative offset
    "rate(http_requests_total[5m] offset -5m)",
    // Complex nested expression
    r#"sum by (job) (rate(http_requests_total{status=~"2.."}[5m])) / sum by (job) (rate(http_requests_total[5m]))"#,
];

/// Alert-style expressions (commonly used in recording rules and alerts)
pub const ALERT_EXPRESSIONS: &[&str] = &[
    // High error rate
    r#"sum(rate(http_requests_total{status=~"5.."}[5m])) by (job) / sum(rate(http_requests_total[5m])) by (job) > 0.1"#,
    // High latency
    r#"histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le, job)) > 0.5"#,
    // Instance down
    "up == 0",
    // Disk almost full
    r#"(node_filesystem_avail_bytes / node_filesystem_size_bytes) < 0.1"#,
    // High memory usage
    r#"(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes > 0.9"#,
    // High CPU
    r#"100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80"#,
    // Request spike (using rate of change)
    r#"deriv(rate(http_requests_total[5m])[30m:1m]) > 10"#,
    // Absent metric (service down)
    r#"absent(up{job="critical-service"})"#,
    // Changes detection
    "changes(some_config_metric[1h]) > 0",
    // Resets detection (counter reset indicates restart)
    "resets(http_requests_total[1h]) > 0",
];

/// Recording rule expressions (pre-computed metrics)
pub const RECORDING_RULES: &[&str] = &[
    // Request rate
    "sum(rate(http_requests_total[5m])) by (job)",
    // Error rate
    r#"sum(rate(http_requests_total{status=~"5.."}[5m])) by (job)"#,
    // Latency quantiles
    r#"histogram_quantile(0.50, sum(rate(http_request_duration_seconds_bucket[5m])) by (le, job))"#,
    r#"histogram_quantile(0.90, sum(rate(http_request_duration_seconds_bucket[5m])) by (le, job))"#,
    r#"histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le, job))"#,
    // Resource usage
    r#"avg by (instance) (irate(node_cpu_seconds_total{mode!="idle"}[5m]))"#,
    "(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes",
    // Aggregated metrics
    "sum(container_memory_usage_bytes) by (namespace)",
    "count(up) by (job)",
];

/// Edge cases and corner cases for robustness testing
pub const EDGE_CASES: &[&str] = &[
    // Very nested expressions
    "sum(avg(min(max(some_metric))))",
    // Multiple operations
    "a + b - c * d / e % f ^ g",
    // Deeply nested parentheses
    "(((((some_metric)))))",
    // Multiple negations
    "----some_metric",
    // Complex label matchers
    r#"metric{a="1",b="2",c="3",d="4",e="5",f="6",g="7",h="8",i="9",j="10"}"#,
    // Long metric names
    "this_is_a_very_long_metric_name_that_might_test_parser_limits",
    // Unicode in labels
    r#"metric{label="中文"}"#,
    r#"metric{label="日本語"}"#,
    r#"metric{label="한국어"}"#,
    // Special characters in regex
    r#"metric{label=~".*\\.example\\.com"}"#,
    r#"metric{label=~"^(foo|bar|baz)$"}"#,
    // Empty label value
    r#"metric{label=""}"#,
    // Metric with all numeric suffix
    "metric_123",
    "metric_1_2_3",
    // Underscores
    "_metric",
    "__metric__",
    "metric_",
    // Colons in metric name (Prometheus federation)
    "job:metric:rate5m",
    "namespace:container_cpu:rate5m",
    // Very long duration
    "some_metric[1000000s]",
    // Very small duration
    "some_metric[1ms]",
    // Combination of all modifiers
    r#"some_metric{job="foo"}[5m] offset 1h @ start()"#,
    // Complex subquery
    "avg_over_time(sum_over_time(rate(metric[5m])[1h:])[1d:1h])",
];

/// Whitespace and formatting variations
pub const WHITESPACE_VARIATIONS: &[&str] = &[
    // No spaces
    "sum(rate(http_requests_total[5m]))by(job)",
    // Extra spaces
    "sum  (  rate  (  http_requests_total  [  5m  ]  )  )  by  (  job  )",
    // Newlines
    "sum(\n  rate(\n    http_requests_total[5m]\n  )\n) by (job)",
    // Tabs
    "sum(\trate(\thttp_requests_total[5m]\t)\t) by (job)",
    // Mixed whitespace
    "sum( \t\n rate( \t\n http_requests_total[5m] \t\n ) \t\n ) by (job)",
];

/// Comment handling (if supported - PromQL standard doesn't have comments)
/// Some implementations might support line comments
pub const WITH_COMMENTS: &[&str] = &[
    // Standard PromQL doesn't have comments, but some tools might
    // "some_metric # this is a comment",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_world_queries() {
        // Verify real-world query test data
        for query in REAL_WORLD_QUERIES {
            assert!(!query.is_empty(), "Empty query in REAL_WORLD_QUERIES");
        }
    }

    #[test]
    fn test_alert_expressions() {
        // Verify alert expression test data
        for expr in ALERT_EXPRESSIONS {
            assert!(!expr.is_empty(), "Empty expression in ALERT_EXPRESSIONS");
        }
    }

    #[test]
    fn test_edge_cases() {
        // Verify edge case test data
        for case in EDGE_CASES {
            assert!(!case.is_empty(), "Empty case in EDGE_CASES");
        }
    }
}
