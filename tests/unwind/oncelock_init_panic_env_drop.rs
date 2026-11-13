// A by-value init closure for OnceLock::get_or_init drops its moved env
// while unwinding, as through any FnOnce call.
struct Loud(&'static str);
impl Drop for Loud {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn call<F: FnOnce() -> u32>(f: F) -> u32 { f() }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let owned = Loud("moved env");
    let r = std::panic::catch_unwind(move || call(move || { let _o = &owned; panic!("x") }));
    println!("{}", r.is_err());
    let owned = Loud("moved env 2");
    let r = std::panic::catch_unwind(move || std::sync::OnceLock::<u32>::new().get_or_init(move || { let _o = &owned; panic!("x") }).clone());
    println!("{}", r.is_err());
    let owned = Loud("moved env 3");
    let r = std::panic::catch_unwind(move || *std::sync::OnceLock::<u32>::new().get_or_init(move || { let o = owned; if o.0.len() > 0 { panic!("x") } 1 }));
    println!("{}", r.is_err());
}
