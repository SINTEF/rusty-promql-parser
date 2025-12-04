use afl::fuzz;
use rusty_promql_parser::expr;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = expr(s);
        }
    });
}
