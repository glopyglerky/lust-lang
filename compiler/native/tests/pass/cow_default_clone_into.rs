// Cow's Default (Owned of the Owned type's default), its own
// to_owned (the blanket Clone impl: a Borrowed stays Borrowed),
// clone_into on str and Copy slices with buffer reuse, and the
// generic AsRef and Borrow bounds at a Cow.
use std::borrow::{Borrow, Cow};
use std::path::Path;

#[derive(Clone, Default, Debug)]
struct D {
    n: i32,
}

#[derive(Default)]
struct Holder<'a> {
    name: Cow<'a, str>,
    data: Cow<'a, [u8]>,
}

fn len_of<T: Borrow<str> + ?Sized>(t: &T) -> usize {
    t.borrow().len()
}
fn asref_len<T: AsRef<str> + ?Sized>(t: &T) -> usize {
    t.as_ref().len()
}
fn mk_default<T: Default>() -> T {
    T::default()
}

fn main() {
    let a: Cow<str> = Cow::default();
    let b: Cow<[u8]> = Cow::<[u8]>::default();
    let c: Cow<D> = Default::default();
    let p: Cow<Path> = Cow::default();
    println!("{:?} {:?} {:?} {:?}", a, b, c, p);
    println!("{} {}", matches!(a, Cow::Owned(_)), matches!(b, Cow::Owned(_)));
    let h: Holder = Holder::default();
    println!("{:?} {:?}", h.name, h.data);
    let g: Cow<str> = mk_default();
    println!("{:?}", g);

    let c1: Cow<str> = Cow::Borrowed("x");
    let c2: Cow<str> = c1.to_owned();
    println!("{} {}", c2, matches!(c2, Cow::Borrowed(_)));
    let c3: Cow<str> = Cow::Owned("y".to_string());
    println!("{}", matches!(c3.to_owned(), Cow::Owned(_)));

    let mut st = String::with_capacity(32);
    st.push_str("oldold");
    "new".clone_into(&mut st);
    println!("{} {}", st, st.capacity());
    let mut st2 = String::from("x");
    "a longer replacement".clone_into(&mut st2);
    println!("{} {}", st2, st2.capacity());
    let mut v: Vec<u8> = Vec::with_capacity(16);
    v.extend_from_slice(&[9, 9, 9]);
    [7u8][..].clone_into(&mut v);
    println!("{:?} {}", v, v.capacity());
    let mut v2 = vec![1u8];
    [5u8, 6, 7, 8, 9][..].clone_into(&mut v2);
    println!("{:?} {}", v2, v2.capacity());

    let cc: Cow<str> = Cow::Owned("abcd".to_string());
    println!("{} {}", len_of(&cc), asref_len(&cc));
    let r: &str = cc.as_ref();
    println!("{}", r);
}
