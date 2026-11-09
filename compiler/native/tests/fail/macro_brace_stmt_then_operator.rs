// expect: L0110
// A `{}` macro call at statement start is a statement: its `match`
// expansion must be `()`, and `- 1` is the next statement (rustc E0308).
macro_rules! m {
    ($e:expr) => {
        match $e {
            1 => 1,
            _ => 2,
        }
    };
}

fn f(x: i32) -> i32 {
    m! {x} - 1
}

fn main() {
    println!("{}", f(1));
}
