// The payload type of each std panic site: &str when Rust's message is a
// literal (after inlining literal format arguments), else String.
use std::panic;
use std::cell::RefCell;
fn kind(name: &str, f: impl FnOnce() + panic::UnwindSafe) {
    let e = panic::catch_unwind(f).unwrap_err();
    let k = if e.downcast_ref::<&str>().is_some() { "str" } else if e.downcast_ref::<String>().is_some() { "String" } else { "other" };
    println!("{name}: {k}");
}
fn n() -> i32 { "0".len() as i32 - 1 }
fn main() {
    panic::set_hook(Box::new(|_| {}));
    kind("lit", || panic!("x"));
    kind("lit1arg", || panic!("x {}", 1));
    kind("litstr", || panic!("x {}", "s"));
    kind("litchar", || panic!("x {}", 'c'));
    kind("litbool", || panic!("x {}", true));
    kind("litneg", || panic!("x {}", -1));
    kind("litfloat", || panic!("x {}", 1.5));
    kind("litu8", || panic!("x {}", 1u8));
    kind("litdbg", || panic!("x {:?}", "s"));
    kind("litwidth", || panic!("x {:>3}", 1));
    kind("var", || { let a = 1; panic!("x {}", a) });
    kind("inline", || { let a = 1; panic!("x {a}") });
    kind("const", || { const C: i32 = 3; panic!("x {}", C) });
    kind("add", || { let _ = i32::MAX + n(); let _ = (i32::MAX - n()) + 1; });
    kind("div0", || { let _ = 1 / n(); });
    kind("rem0", || { let _ = 1 % n(); });
    kind("neg", || { let _ = -(i32::MIN + n()); });
    kind("shl", || { let _ = 1i32 << (40 + n()); });
    kind("index", || { let v = vec![1]; let _ = v[n() as usize + 3]; });
    kind("arrindex", || { let v = [1]; let _ = v[n() as usize + 3]; });
    kind("unwrap", || { let o: Option<i32> = None; o.unwrap(); });
    kind("expect", || { let o: Option<i32> = None; o.expect("e"); });
    kind("runwrap", || { let o: Result<i32, i32> = Err(1); o.unwrap(); });
    kind("rexpect", || { let o: Result<i32, i32> = Err(1); o.expect("m"); });
    kind("unwraperr", || { let o: Result<i32, i32> = Ok(1); o.unwrap_err(); });
    kind("refcell", || { let c = RefCell::new(1); let _a = c.borrow_mut(); let _b = c.borrow(); });
    kind("refcellmut", || { let c = RefCell::new(1); let _a = c.borrow(); let _b = c.borrow_mut(); });
    kind("slice", || { let v = vec![1]; let _ = &v[n() as usize + 2..]; });
    kind("sliceend", || { let v = vec![1]; let _ = &v[..n() as usize + 2]; });
    kind("strslice", || { let s = "é"; let _ = &s[..1 + n() as usize]; });
    kind("remove", || { let mut v = vec![1]; v.remove(n() as usize + 3); });
    kind("insert", || { let mut v = vec![1]; v.insert(n() as usize + 3, 1); });
    kind("assert", || assert!(n() > 1));
    kind("assertmsg", || assert!(n() > 1, "m"));
    kind("assertmsgfmt", || assert!(n() > 1, "m {}", n()));
    kind("asserteq", || assert_eq!(n(), 1));
    kind("unreachable", || unreachable!());
    kind("unreachablemsg", || unreachable!("m {}", 1));
    kind("todo", || todo!());
    kind("unimpl", || unimplemented!());
    kind("swap", || { let mut v = vec![1]; v.swap(0, 3 + n() as usize); });
    kind("chardigit", || { let _ = 'a'.to_digit(40 + n() as u32); });
    kind("repeatcap", || { let _ = "ab".repeat(usize::MAX - n() as usize); });
    kind("withcap", || { let _ = Vec::<u64>::with_capacity(usize::MAX - n() as usize); });
    kind("pow", || { let _ = 10i32.pow(20 + n() as u32); });
    kind("stepby", || { let _ = (0..3).step_by(n() as usize); });
}
