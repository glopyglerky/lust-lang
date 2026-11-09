// `Box<dyn FnOnce>` calls consume the box: an FnOnce env moves out, an
// Fn env drops after the call, and an uncalled box drops its env.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn run(f: Box<dyn FnOnce() -> String>) -> String {
    f()
}
fn consume() {
    let d = D("moved");
    let s = String::from("text");
    let f: Box<dyn FnOnce() -> String> = Box::new(move || {
        let t = s + "!";
        drop(d);
        t
    });
    println!("{}", run(f));
}
fn by_ref_env() {
    let keep = D("kept");
    let h: Box<dyn FnOnce() -> usize> = Box::new(move || keep.0.len());
    println!("{}", h());
}
fn main() {
    consume();
    let g: Box<dyn FnOnce(u8) -> u8> = Box::new(|x| x + 1);
    println!("{}", g(4));
    by_ref_env();
    let never: Box<dyn FnOnce()> = Box::new(|| println!("never"));
    drop(never);
    let e = D("env");
    let unused: Box<dyn FnOnce()> = Box::new(move || drop(e));
    let _ = &unused;
    println!("end");
}
