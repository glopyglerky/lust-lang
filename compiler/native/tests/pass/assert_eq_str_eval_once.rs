// assert_eq!/assert_ne! between a String and a &str evaluate each operand once
// (tf-build: the str comparison lowered both operands a second time, so a
// call with side effects ran twice, and a caught panic printed twice).
use std::cell::Cell;

thread_local! {
    static N: Cell<u32> = Cell::new(0);
}

fn s() -> String {
    N.with(|n| n.set(n.get() + 1));
    "a".to_string()
}

fn count() -> u32 {
    N.with(|n| n.get())
}

fn text(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
    let p = std::panic::catch_unwind(f).expect_err("no panic");
    match p.downcast::<String>() {
        Ok(s) => *s,
        Err(_) => "not a String".into(),
    }
}

fn main() {
    assert_eq!(s(), "a");
    println!("{}", count());
    assert_eq!("a", s());
    println!("{}", count());
    assert_ne!(s(), "b");
    println!("{}", count());
    assert_eq!(s(), s());
    println!("{}", count());
    let r = &s();
    assert_eq!(r, "a");
    assert_eq!(&&s(), &"a");
    println!("{}", count());
    let cell = Cell::new(0);
    let next = || {
        cell.set(cell.get() + 1);
        format!("v{}", cell.get())
    };
    assert_eq!(next(), "v1");
    assert_ne!(next(), "v1");
    println!("{}", cell.get());
    std::panic::set_hook(Box::new(|_| eprintln!("hook")));
    assert_eq!(text(|| std::panic::panic_any(String::from("x"))), "x");
}
