// feature: panic.recovery
// form: catch_unwind with AssertUnwindSafe, as in ts-rust session and emit recovery
// rustc-panic: unwind
fn main() {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { panic!("boom"); }));
    println!("caught={} panicking={}", r.is_err(), std::thread::panicking());
}
