// expect: L0612
fn call<F: FnMut() -> String>(mut f: F) {
    f();
}
fn main() {
    let s = String::from("x");
    call(move || s);
}
