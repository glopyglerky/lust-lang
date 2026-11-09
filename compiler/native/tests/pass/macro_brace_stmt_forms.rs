// `{}` macro calls at statement start: a `.` continues the call's value
// (rustc parses `m!{x}.max(3)` as one expression), and a `let`
// expansion ends its statement, so `- 1` after it is the block's value.
macro_rules! m {
    ($e:expr) => {
        match $e {
            1 => 1,
            _ => 2,
        }
    };
}
macro_rules! s {
    ($e:expr) => {
        let _q = $e;
    };
}

fn f(x: i32) -> i32 {
    m! {x}.max(3)
}

fn g(x: i32) -> i32 {
    s! {x} - 1
}

fn main() {
    println!("{} {} {}", f(1), f(7), g(5));
}
