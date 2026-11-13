// `Clone::clone_from` on a Cow reuses the Owned buffer (R24).
use std::borrow::Cow;

fn main() {
    let mut a: Cow<str> = Cow::Owned("long string here".to_string());
    let b: Cow<str> = Cow::Borrowed("b");
    a.clone_from(&b);
    println!("{}", a);

    let mut c: Cow<str> = Cow::Owned(String::with_capacity(64));
    if let Cow::Owned(s) = &mut c {
        s.push_str("abc");
    }
    c.clone_from(&Cow::Owned("hello".to_string()));
    match &c {
        Cow::Owned(s) => println!("owned cap={} {:?}", s.capacity(), s),
        Cow::Borrowed(s) => println!("borrowed {:?}", s),
    }

    let mut d: Cow<[i32]> = Cow::Owned(vec![1, 2, 3]);
    let src: Cow<[i32]> = Cow::Borrowed(&[7, 8][..]);
    d.clone_from(&src);
    println!("{:?}", d);
}
