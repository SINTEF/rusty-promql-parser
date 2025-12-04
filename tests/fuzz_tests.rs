use rusty_promql_parser::expr;

#[test]
fn test_first_crash() {
    let input = "deriv(rate(http_requests_total[5555555555555555555m])[30m:1m]) >";
    let result = expr(input);
    assert!(
        result.is_err(),
        "Expected parsing to fail for input: '{}'",
        input
    );
}
