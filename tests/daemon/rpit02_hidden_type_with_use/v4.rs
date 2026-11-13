// impl Trait hidden type edits in a file with a `use` item (the resolver
// path): only the definer bodies change; the callers must see each new
// hidden type (hunt2 daemon_rpit_hidden_type_stale).
use std::fmt::Display;
fn compose(a: u32) -> impl Fn(u32) -> u32 { let t = (9u64, String::from("abc"), 1u8); move |x| x + a + t.0 as u32 + t.1.len() as u32 + t.2 as u32 }
fn main() {
    let h = compose(5);
    println!("{}", h(3));
}
