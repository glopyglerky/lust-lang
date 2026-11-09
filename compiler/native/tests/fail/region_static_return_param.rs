// expect: L0733
// rustc: lifetime may not live long enough (the parameter's lifetime is not 'static).
fn get(v: &Vec<String>) -> &'static str {
    &v[0]
}

fn main() {
    let v = vec![String::from("a")];
    println!("{}", get(&v));
}
