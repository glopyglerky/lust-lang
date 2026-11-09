// expect: L0110
// rustc E0308: a fn pointer of one argument is not one of two.
struct Two<T>(fn(T, T) -> T);

fn one(x: u8) -> u8 {
    x
}

fn main() {
    let t = Two(one);
    println!("{}", (t.0)(1, 2));
}
