// Closure operators whose operand type comes from a later call or an
// Fn bound: the result is the operator impl's output once the operand
// is known (a literal falls back to i32).
fn apply<F: Fn(u8) -> u8>(f: F) -> u8 {
    f(3)
}

fn main() {
    let c = |x| x + 1;
    println!("{}", c(5));
    let d = |x| x * 2;
    let r: u64 = d(7u64);
    println!("{}", r);
    println!("{}", apply(|x| x + 1));
    let e = |a, b| a - b;
    println!("{}", e(9i16, 4));
    let g = |x| 1 + x;
    let s: i64 = g(2i64);
    println!("{}", s);
}
