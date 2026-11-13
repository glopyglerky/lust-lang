// expect: L0305
fn take(s: String) -> bool { s.len() > 1 }
enum E { A(String), B(String) }
fn main() {
    let e = E::A("hi".to_string());
    match e {
        E::A(x) | E::B(x) if take(x) => println!("y"),
        _ => println!("n"),
    }
}
