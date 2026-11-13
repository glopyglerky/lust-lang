// A closure that captures nothing coerces to a fn pointer.
fn main() {
    let f: fn(i32) -> i32 = |x| x + 1;
    println!("{}", f(1));
}
