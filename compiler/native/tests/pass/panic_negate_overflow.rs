// Negating a MIN value held in a variable still panics at run time.
// panic: attempt to negate with overflow
fn neg(x: i8) -> i8 {
    -x
}

fn main() {
    println!("before");
    let m: i8 = -128;
    println!("{}", neg(m));
}
