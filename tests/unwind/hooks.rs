// set_hook, take_hook, restoring the taken hook, the default hook text
// after take_hook, PanicHookInfo's payload, location, and Display, and a
// panic inside the hook (Rust aborts: "thread panicked while processing panic").
use std::panic;
fn text(p: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = p.downcast_ref::<&str>() { s.to_string() } else if let Some(s) = p.downcast_ref::<String>() { s.clone() } else { String::from("?") }
}
fn main() {
    panic::set_hook(Box::new(|info| {
        let loc = info.location().unwrap();
        println!("HOOK [{}] at {}:{}:{} | {} | {:?}", text(info.payload()), loc.file(), loc.line(), loc.column(), info, info.payload_as_str());
    }));
    let r = panic::catch_unwind(|| { panic!("a {}", 1); });
    println!("caught {}", r.is_err());
    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_| { println!("quiet"); }));
    let _ = panic::catch_unwind(|| { panic!("b"); });
    panic::set_hook(hook);
    let _ = panic::catch_unwind(|| { let v: Vec<u8> = vec![]; v[3] });
    let _ = panic::catch_unwind(|| { panic::panic_any(7u8) });
    let _ = panic::take_hook();
    let _ = panic::catch_unwind(|| { panic!("c"); });
    let _ = panic::catch_unwind(|| { panic!("d {}", 4); });
    panic::set_hook(Box::new(|_| { panic!("in hook"); }));
    panic!("e");
}
