// Cow methods: into_owned (exact capacity), as_ref, borrow, to_mut on
// both variants; Cow<[u8]>; HashSet<Cow<str>> looked up by &str;
// ordering compares the str values.
use std::borrow::{Borrow, Cow};
use std::collections::HashSet;

fn fix(s: &str) -> Cow<'_, str> {
    if s.starts_with("x") {
        Cow::Owned(format!("y{}", s))
    } else {
        Cow::Borrowed(s)
    }
}

fn bytes(b: &[u8]) -> Cow<'_, [u8]> {
    if b.len() > 2 {
        Cow::Owned(b[..2].to_vec())
    } else {
        Cow::Borrowed(b)
    }
}

fn main() {
    let a = fix("abc");
    let b = fix("xyz");
    let ao: String = a.clone().into_owned();
    let bo: String = b.clone().into_owned();
    println!("{} {} {} {}", ao, ao.capacity(), bo, bo.capacity());
    let r: &str = a.as_ref();
    let r2: &str = b.borrow();
    println!("{} {}", r, r2);
    let mut c = fix("abc");
    c.to_mut().push_str("!");
    println!("{} {:?}", c, matches!(c, Cow::Owned(_)));
    let mut d = fix("xa");
    d.to_mut().push('?');
    println!("{}", d);
    let (raw1, raw2) = ([1u8, 2, 3], [9u8]);
    let v = bytes(&raw1);
    let w = bytes(&raw2);
    let vo = v.into_owned();
    println!("{:?} {} {:?}", vo, vo.capacity(), w.as_ref());
    let mut names: HashSet<Cow<'_, str>> = HashSet::new();
    names.insert(Cow::Borrowed("k"));
    names.insert(Cow::Owned("j".to_string()));
    println!("{} {} {}", names.contains("k"), names.contains("j"), names.contains("z"));
    println!("{}", names.insert(Cow::Owned("k".to_string())));
    let list: Vec<Cow<'_, str>> = vec![fix("b"), fix("xa"), fix("a")];
    println!("{:?} {} {}", list, list[0] < list[1], list[1] >= list[2]);
}
