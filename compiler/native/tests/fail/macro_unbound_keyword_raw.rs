// expect: L0806
// rustc: no rules expected `r#if`
// An unbound `$if` in a transcriber is written back as `$ r#if`, a raw
// identifier that no longer matches the keyword `if`.
macro_rules! m { ($d:tt if) => { () }; }
macro_rules! o { () => { m!($ if) }; }
fn main() { o!(); }
