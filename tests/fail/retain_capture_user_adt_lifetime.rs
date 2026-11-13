// expect: L0306
// A user struct with a lifetime parameter can store the predicate's
// capture through `&T` (a RefCell inside), so the capture must outlive the
// element's lifetime: rustc rejects this (lifetime may not live long enough).
use std::cell::RefCell;

struct Holder<'a> {
    v: RefCell<Vec<&'a str>>,
}

fn prune<'a>(v: &mut Vec<Holder<'a>>, cap: &str) {
    v.retain(|h| {
        h.v.borrow_mut().push(cap);
        true
    });
}

fn main() {
    let mut v = vec![Holder { v: RefCell::new(vec![]) }];
    prune(&mut v, "x");
    println!("{}", v.len());
}
