// expect: L0001
// rustc: labels cannot use keyword names, even in a fn nothing calls.
fn unused() { 'fn: loop { break 'fn; } }
fn main() {}
