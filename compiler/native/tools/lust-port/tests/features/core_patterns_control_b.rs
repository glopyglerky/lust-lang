// feature: core.patterns_control
// form: if-let chain with let chains in one condition
fn probe(a: Option<i32>, b: Option<i32>) -> i32 {
    if let Some(x) = a && let Some(y) = b && x > 0 { x + y } else { 0 }
}
fn main() { println!("{} {}", probe(Some(1), Some(2)), probe(None, Some(2))); }
