// expect: L0307
fn dangle(n: &i64) -> &String {
    let s = format!("{}", n);
    &s
}

fn main() {
    println!("{}", dangle(&1));
}
