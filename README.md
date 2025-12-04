# Rusty PromQL Parser

Rust port of the Prometheus [PromQL parser](https://github.com/prometheus/prometheus/tree/main/promql/parser) using the [nom](https://github.com/rust-bakery/nom) parser combinator library.

## Examples

### A metric with label filtering

```rust
use rusty_promql_parser::expr;

let input = r#"go_gc_duration_seconds{instance="localhost:9090", job="alertmanager"}"#;
let (rest, ast) = expr(input).expect("failed to parse");
assert!(rest.is_empty());
println!("{:#?}", ast);
```

```text
VectorSelector {
    name: Some("go_gc_duration_seconds"),
    matchers: [
        LabelMatcher { name: "instance", op: Equal, value: "localhost:9090" },
        LabelMatcher { name: "job", op: Equal, value: "alertmanager" }
    ],
}
```

### Aggregation operators

```rust
use rusty_promql_parser::expr;

let input = r#"sum by (app, proc) (
  instance_memory_limit_bytes - instance_memory_usage_bytes
) / 1024 / 1024"#;
let (rest, ast) = expr(input).expect("failed to parse");
assert!(rest.is_empty());
println!("{:#?}", ast);
```

```text
BinaryExpr {
    op: Div,
    lhs: BinaryExpr {
        op: Div,
        lhs: Aggregation {
            op: "sum",
            expr: BinaryExpr {
                op: Sub,
                lhs: VectorSelector { name: Some("instance_memory_limit_bytes"), ... },
                rhs: VectorSelector { name: Some("instance_memory_usage_bytes"), ... },
            },
            grouping: Some(Grouping { action: By, labels: ["app", "proc"] })
        },
        rhs: Number(1024.0),
    },
    rhs: Number(1024.0),
}
```

## ⚠️ Vibecoded ⚠️

This project is mostly vibecoded, using the official [Prometheus PromQL parser](https://github.com/prometheus/prometheus/tree/main/promql) (Apache 2.0) and a [Rust port by HewlettPackard](https://github.com/HewlettPackard/prometheus-parser-rs) (MIT) as reference. You are welcome.

## Testing

The advanced stochastic parrots were requested to import the test cases from the original Prometheus parser to ensure some compatibility.

In addition to the unit tests, we run some AFL fuzzing to ensure robustness against malformed inputs. One crash was found and fixed during development: a number overflow panic when dealing with long durations and unit conversions.

This is not perfect, but unit tests, fuzzing, nom combinators, and Rust, should make this parser reasonably robust.

## Why?

The main goal was to experiment whether vibecoding technology of December 2025 could allow one to port a non-trivial piece of software from Golang to Rust, in a reasonable time frame. Apparently, yes. It took a few hours.

## You may not want to use this in production

As stated in the license, this is provided as-is, without warranty of any kind. It is also vibecoded.

But it's also relatively well tested and based on solid foundations with nom and rust, and of course the original Prometheus parser and its exhaustive test suite.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
