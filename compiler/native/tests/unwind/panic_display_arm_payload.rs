// std's `panic!` has its own arm for exactly `panic!("{}", arg)` (also an
// `assert!` message): `panic_display(&arg)` formats `*arg`, so a literal argument
// is not inlined and the payload is a `String`. Other format strings inline a
// literal argument, and a message with no runtime argument is a `&'static str`.
fn kind(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
    let p = std::panic::catch_unwind(f).expect_err("no panic");
    if let Some(s) = p.downcast_ref::<String>() {
        format!("String {s}")
    } else if let Some(s) = p.downcast_ref::<&str>() {
        format!("&str {s}")
    } else {
        "other".into()
    }
}

macro_rules! inner {
    () => {
        panic!("{}", "m")
    };
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let x = 4;
    println!("{}", kind(|| panic!("lit")));
    println!("{}", kind(|| panic!("{}", "a")));
    println!("{}", kind(|| panic!("{}", "a",)));
    println!("{}", kind(|| panic!("{}", 5)));
    println!("{}", kind(|| panic!("{}", &"r")));
    println!("{}", kind(|| panic!("a{}b", "c")));
    println!("{}", kind(|| panic!("{0}", "a")));
    println!("{}", kind(|| panic!(r"{}", "a")));
    println!("{}", kind(|| panic!(concat!("{", "}"), "a")));
    println!("{}", kind(|| panic!("{}", x)));
    println!("{}", kind(|| assert!(x == 5, "{}", "z")));
    println!("{}", kind(|| assert!(x == 5, "m {}", "z")));
    println!("{}", kind(|| inner!()));
}
