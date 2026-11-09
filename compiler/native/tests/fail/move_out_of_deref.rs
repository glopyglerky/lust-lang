// expect: L0305
fn take(s: &String) -> String {
    *s
}

fn main() {
    println!("{}", take(&String::from("a")));
}
