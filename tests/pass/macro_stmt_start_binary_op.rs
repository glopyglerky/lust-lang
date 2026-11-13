// A `()` or `[]` macro call at the start of a statement is an
// expression: an operator after it continues it, also when the macro
// expands to a block-like form (`match`, `if`, a block). A `{}` call
// there is a whole statement.
macro_rules! is_one {
    ($e:expr) => {
        match $e {
            1 => true,
            _ => false,
        }
    };
}
macro_rules! blk {
    ($e:expr) => {{ $e * 2 }};
}
macro_rules! when {
    ($c:expr, $a:expr, $b:expr) => {
        if $c { $a } else { $b }
    };
}

fn a(x: u8) -> bool {
    is_one!(x) || x > 5
}

fn b(x: u8) -> bool {
    matches!(x, 1) || (x > 5 && matches!(x, 7))
}

fn c(x: i32) -> i32 {
    blk!(x) + 1
}

fn d(x: i32) -> i32 {
    when!(x > 0, x, -x) * 10
}

fn e(x: u8) -> bool {
    is_one![x] && true
}

fn main() {
    println!("{} {} {} {}", a(1), a(3), a(7), b(7));
    println!("{} {} {} {}", b(3), c(4), d(-3), e(1));
}
