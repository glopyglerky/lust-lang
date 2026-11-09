// capability: rejected: variadic parameters (rustc rejects them outside `extern` blocks too)
// expect: L0820

fn f(x: u32, ..) {}

fn main() {}
