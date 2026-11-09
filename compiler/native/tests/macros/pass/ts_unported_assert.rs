// macro: unported, go_assert
// source: crates/ts_goport/src/gopanic.rs:41
// source: crates/ts_goport/src/gostd/debug.rs:71
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
// Stand-ins at the exact $crate paths the macros name.
pub mod core {
    pub fn record_unported(name: &str) { println!("recorded:{name}"); }
}
pub mod gostd {
    pub mod debug {
        pub fn assert_failed(msg: Option<std::fmt::Arguments<'_>>) -> ! {
            match msg {
                Some(m) => panic!("assertion failed: {m}"),
                None => panic!("assertion failed"),
            }
        }
    }
}

#[macro_export]
macro_rules! unported {
    ($go_name:expr) => {{
        $crate::core::record_unported($go_name);
        panic!("unported Go code: {}", $go_name)
    }};
}

#[macro_export]
macro_rules! go_assert {
    ($cond:expr $(,)?) => {
        if !$cond {
            $crate::gostd::debug::assert_failed(None)
        }
    };
    ($cond:expr, $($arg:tt)+) => {
        if !$cond {
            $crate::gostd::debug::assert_failed(Some(format_args!($($arg)+)))
        }
    };
}

fn panic_text(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
    std::panic::set_hook(Box::new(|_| {}));
    match std::panic::catch_unwind(f) {
        Ok(()) => "no panic".to_string(),
        Err(e) => {
            if let Some(s) = e.downcast_ref::<String>() { s.clone() }
            else if let Some(s) = e.downcast_ref::<&str>() { s.to_string() }
            else { "unknown".to_string() }
        }
    }
}

fn main() {
    go_assert!(1 + 1 == 2);
    go_assert!(true, "unused {}", 9);
    println!("assert-true ok");
    println!("assert-false: {}", panic_text(|| go_assert!(1 == 2, "one is not {}", 2)));
    println!("unported: {}", panic_text(|| { unported!("checker.go:Foo"); }));
}
