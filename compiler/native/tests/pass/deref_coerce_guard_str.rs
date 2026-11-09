// A reference to a guard (Ref, RefMut, a lock guard) or a counted
// pointer over a String coerces to &str through the guard's Deref, then
// String's. lust once passed the guard as the String (garbage length).
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

fn need(s: &str) -> usize {
    s.len()
}

fn back<'a>(g: &'a std::cell::Ref<'_, String>) -> &'a str {
    g
}

struct Holder<'a> {
    s: &'a str,
}

fn main() {
    let c = RefCell::new(String::from("abcd"));
    {
        let r = c.borrow();
        println!("{} {}", need(&r), back(&r));
        let s: &str = &r;
        println!("{s}");
        let rr = &&r;
        let h = Holder { s: rr };
        println!("{}", h.s);
    }
    {
        let mut w = c.borrow_mut();
        w.push('e');
        let rw = &mut w;
        println!("{} {}", need(rw), *w);
    }
    let m = Mutex::new(String::from("mutex"));
    println!("{}", need(&m.lock().unwrap()));
    let l = RwLock::new(String::from("rw"));
    {
        let g = l.read().unwrap();
        let s: &str = if need(&g) > 1 { &g } else { "none" };
        println!("{s}");
    }
    {
        let mut g = l.write().unwrap();
        g.make_ascii_uppercase();
        println!("{}", need(&&g));
    }
    println!("{}", *l.read().unwrap());
    let a = Arc::new(String::from("arc"));
    let b = Rc::new(String::from("rc"));
    let v: [&str; 2] = [&a, &b];
    println!("{v:?}");
}
