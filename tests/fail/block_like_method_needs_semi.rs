// expect: L0001
// rustc: `match .. {}.clone()` is no longer block-like, so the statement
// needs a `;` before the next one ("expected `;`, found keyword `if`").
fn main() {
    let c = true;
    match c { _ => () }.clone()
    if c { println!("x"); }
}
