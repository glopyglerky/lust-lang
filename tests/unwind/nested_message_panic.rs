// A panic inside a panic message's Display aborts (Rust formats the message
// inside the panic), after an earlier caught panic.
struct Bad;
impl std::fmt::Display for Bad { fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result { panic!("inner {}", 1 + "ab".len()) } }
struct D;
impl Drop for D { fn drop(&mut self) { println!("drop D"); } }
fn main() {
    let r = std::panic::catch_unwind(|| { let _d = D; panic!("ok {}", "x".len()) });
    println!("{}", r.is_err());
    let _d = D;
    panic!("outer {}", Bad);
}
