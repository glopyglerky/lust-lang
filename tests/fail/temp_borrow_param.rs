// expect: L0306
fn f(mut v: &mut Vec<i64>) {
    v = &mut Vec::new();
}

fn main() {
    let mut v: Vec<i64> = Vec::new();
    f(&mut v);
    println!("{}", v.len());
}
