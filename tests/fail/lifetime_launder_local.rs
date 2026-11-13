// expect: L0307
fn f(a: &i32) -> &i32 {
    let x = 5;
    let mut r = &x;
    println!("{}", r);
    r = a;
    r
}

fn main() {
    let a = 2;
    println!("{}", f(&a));
}
