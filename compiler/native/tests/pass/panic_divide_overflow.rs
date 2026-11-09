// The MIN literal must reach the callee; the panic is the divide inside `f`.
// panic: attempt to divide with overflow
fn f(a: i32, b: i32) -> i32 {
    a / b
}

fn main() {
    println!("before");
    println!("{}", f(-2147483648, -1));
}
