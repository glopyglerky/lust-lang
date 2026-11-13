// `HashSet::get` returns the receiver's loan around the set's own element:
// a `&'static str` element stays `'static` (the ts-rust scanner's
// `intern_token_value` shape, with a fixed table instead of `Box::leak`).
use std::cell::RefCell;
use std::collections::HashSet;
static TABLE: [&str; 3] = ["ab", "cd", "ef"];
pub fn intern(value: &str) -> &'static str {
    thread_local! {
        static VALUES: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
    }
    if value.is_empty() {
        return "";
    }
    VALUES.with(|values| {
        let mut values = values.borrow_mut();
        if let Some(existing) = values.get(value) {
            return *existing;
        }
        let stored: &'static str = TABLE.iter().copied().find(|t| *t == value).unwrap_or("?");
        values.insert(stored);
        stored
    })
}
fn first(set: &HashSet<&'static str>, v: &str) -> &'static str {
    set.get(v).copied().unwrap_or("")
}
fn main() {
    let a = intern("ab");
    let b = intern(&String::from("ab"));
    let mut s = HashSet::new();
    s.insert("k");
    println!("{} {} {} {}", a, a == b, intern("cd"), first(&s, "k"));
}
