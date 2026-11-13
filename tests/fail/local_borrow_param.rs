// expect: L0306
fn f(mut v: &mut Vec<i64>) {
    let mut local: Vec<i64> = Vec::new();
    v = &mut local;
    println!("{}", v.len());
}

fn main() {
    let mut v: Vec<i64> = Vec::new();
    f(&mut v);
}
