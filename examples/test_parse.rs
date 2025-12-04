use rusty_promql_parser::expr;

fn main() {
    let inputs = [
        "cpu_temperature{host=\"server1\"}",
        "{__name__=\"abc\", host=\"localhost\"} offset 5m",
        "some_metric[5m:1m] @ 1609459200 offset 10m",
        "some_metric[5m:1m] offset 10m @ 1609459200",
    ];

    for input in inputs {
        println!("\nParsing: {}", input);
        match expr(input) {
            Ok((remaining, parsed)) => {
                println!("  OK! Remaining: '{}'", remaining);
                println!("  Parsed: {:?}", parsed);
            }
            Err(e) => println!("  Error: {:?}", e),
        }
    }
}
