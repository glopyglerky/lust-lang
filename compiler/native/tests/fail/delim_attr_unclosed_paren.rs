// expect: L0001
// rustc: mismatched closing delimiter. The attribute's `(` closes with `]`, in a file with no macro call.
#[allow(dead_code]
fn f() {}

fn main() {}
