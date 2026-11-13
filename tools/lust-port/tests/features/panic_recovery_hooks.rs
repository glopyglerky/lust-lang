// feature: panic.recovery
// form: set_hook/take_hook around a catch, resume_unwind of the payload, as in ts-rust's goport bins
// rustc-panic: unwind
fn main() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|info| println!("hook: {}", info.payload_as_str().unwrap_or("?"))));
    let r = std::panic::catch_unwind(|| panic!("quiet {}", 1));
    std::panic::set_hook(hook);
    let payload = r.unwrap_err();
    println!("{:?}", payload.downcast_ref::<String>());
    let again = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| std::panic::resume_unwind(payload)));
    println!("again err={}", again.is_err());
}
