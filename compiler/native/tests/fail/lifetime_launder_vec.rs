// expect: L0307
fn f(a: &i32) -> Vec<&i32> {
    let x = 5;
    let mut r = &x;
    println!("{}", r);
    r = a;
    let mut v: Vec<&i32> = Vec::new();
    v.push(r);
    v
}

fn main() {
    let a = 2;
    println!("{}", f(&a)[0]);
}
