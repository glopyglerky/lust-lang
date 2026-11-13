// A local reference can be reassigned from different sources when it is only used while
// both sources live. Only a return or a store behind a parameter pins its lifetime.
fn show(a: &i32, b: &i32) -> i32 {
    let mut r = a;
    let first = *r;
    r = b;
    first + *r
}

fn main() {
    let x = 10;
    let y = 20;
    let mut r = &x;
    print!("{} ", r);
    r = &y;
    println!("{}", r);
    println!("{}", show(&x, &y));
}
