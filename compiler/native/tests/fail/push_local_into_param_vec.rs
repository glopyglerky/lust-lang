// expect: L0306
fn f(v: &mut Vec<&i64>) {
    let x: i64 = 1;
    v.push(&x);
}

fn main() {
    let mut v: Vec<&i64> = Vec::new();
    f(&mut v);
    println!("{}", v.len());
}
