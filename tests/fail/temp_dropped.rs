// expect: L0306
fn make() -> String {
    String::from("temp")
}

fn main() {
    let s: &str = make().as_str();
    println!("{}", s);
}
