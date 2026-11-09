// expect: L0308
fn call<F: FnMut()>(f: &F) {
    f();
}
fn main() {
    let mut n = 0;
    let c = || n += 1;
    call(&c);
}
