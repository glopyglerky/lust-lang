// Raw pointers as identity values (w-cinterop): coercion from
// references, core::ptr fns, casts, address equality and hashing,
// as_ptr on Rc, Arc, Vec, slices, and str. No deref anywhere.
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct Key {
    p: *const u8,
    n: i32,
}

fn id_of(x: &i32) -> *const i32 {
    x
}

fn main() {
    let a = 3i32;
    let b = 3i32;
    let p: *const i32 = &a;
    let same = std::ptr::from_ref(&a);
    let eras: *const () = same.cast();
    let null: *const i32 = std::ptr::null();
    let mut m = 4i32;
    let mp: *mut i32 = &mut m;
    let mc: *const i32 = mp;
    println!("{} {} {}", p == same, p != std::ptr::from_ref(&b), eras.addr() == p.addr());
    println!("{} {} {:?}", null.is_null(), mc == id_of(&m), null);
    println!("{} {}", std::ptr::eq(&a, &a), std::ptr::eq(&a, &b));
    println!("{}", (p as usize) == p.expose_provenance() && p.with_addr(p.addr()) == p);

    let rc = Rc::new(String::from("x"));
    let rc2 = rc.clone();
    let arc = Arc::new(9i32);
    let arc2 = arc.clone();
    println!("{} {}", Rc::as_ptr(&rc) == Rc::as_ptr(&rc2), Arc::as_ptr(&arc) == Arc::as_ptr(&arc2));
    println!("{}", Rc::as_ptr(&rc) == std::ptr::from_ref(&*rc));

    let v = vec![10i32, 20, 30];
    let s = String::from("abc");
    println!("{} {}", v.as_ptr() == std::ptr::from_ref(&v[0]), v.as_slice().as_ptr() == v.as_ptr());
    println!("{} {}", s.as_ptr() == s.as_str().as_ptr(), s.as_ptr() == std::ptr::from_ref(&s.as_bytes()[0]));

    let mut counts: HashMap<*const i32, usize> = HashMap::new();
    for x in [&a, &b, &a] {
        *counts.entry(std::ptr::from_ref(x)).or_insert(0) += 1;
    }
    let mut keys: HashSet<Key> = HashSet::new();
    keys.insert(Key { p: v.as_ptr().cast(), n: 1 });
    println!("{} {} {}", counts.len(), counts[&p], keys.contains(&Key { p: v.as_ptr().cast(), n: 1 }));
    println!("{} {}", std::mem::size_of::<*const u8>(), std::mem::size_of::<Option<*const u8>>());

    let gone = {
        let t = 5i32;
        std::ptr::from_ref(&t)
    };
    println!("{}", !gone.is_null());
}
