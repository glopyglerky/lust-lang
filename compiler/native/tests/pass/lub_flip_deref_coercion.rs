// rustc's LUB coercion of match arms, if/else branches and array elements
// when a later site's type is a deref target of the earlier ones (no
// earlier site coerced yet, other than a `&mut T` to `&T` reborrow): the
// earlier sites coerce to it.
// (ts-rust ast/store.rs static_facts_word: `Some(c) => c` with c:
// &Box<[u32]>, then `None => g(v)?` with &[u32].)
use std::rc::Rc;
fn g<'a>(v: &'a [u32]) -> Option<&'a [u32]> {
    Some(v)
}
fn f<'a>(o: Option<&'a Box<[u32]>>, v: &'a [u32]) -> Option<u32> {
    let c = match o {
        Some(c) => c,
        None => g(v)?,
    };
    c.get(1).copied()
}
fn main() {
    let b: Box<[u32]> = vec![1u32, 2, 3].into_boxed_slice();
    println!("{:?} {:?}", f(Some(&b), &[7, 8]), f(None, &[7, 8]));
    let v = vec![1u8, 2, 3];
    let bx: Box<Vec<u8>> = Box::new(vec![4]);
    let s: &[u8] = &[5, 6];
    for k in 0..3 {
        let r = match k {
            0 => &v,
            1 => s,
            _ => &bx,
        };
        println!("{:?}", r);
    }
    let st = String::from("st");
    let bs: Box<str> = "bs".into();
    let rs: Rc<str> = Rc::from("rs");
    let parts = [&bs, &bs];
    let mixed = [&rs, &rs];
    println!("{} {}", parts.len(), mixed.len());
    for k in 0..2 {
        let r = if k == 0 { &bs } else { "lit" };
        let q = if k == 0 { &st } else { &*rs };
        println!("{r} {q}");
    }
    let arr = [&st, "x", &st];
    println!("{:?}", arr);
    let mut v2 = vec![9u8];
    for k in 0..3 {
        let r = match k {
            0 => &mut v2,
            1 => &v,
            _ => s,
        };
        println!("{}", r.len());
    }
    let mut st2 = String::from("m");
    for k in 0..2 {
        let r = if k == 0 { &mut st2 } else { &st };
        println!("{r}");
    }
}
