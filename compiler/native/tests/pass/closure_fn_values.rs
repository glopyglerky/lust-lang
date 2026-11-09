// Closures capture fn pointers and fn items, and take fn pointer parameters.
fn double(x: u8) -> u8 { x * 2 }
fn main() {
    let f: fn(u8) -> u8 = double;
    let g = double;
    let c = || f(2) + g(3);
    println!("{}", c());
    let apply = |h: fn(u8) -> u8, v: u8| h(v);
    println!("{}", apply(double, 5));
    let sz = || std::mem::size_of_val(&f);
    println!("{}", sz());
}
