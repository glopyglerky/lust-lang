// expect: L0524
// rustc error: malformed `should_panic` attribute input (only `expected = "..."`).
#[should_panic(foo = "x")]
fn helper() {}
fn main() {}
