// `value.fmt(f)` on an Adt whose Debug is derived: in an impl of Debug (its own trait is in
// scope), in a free fn with `use std::fmt::Debug`, and through a generic forwarding impl
// (ts-rust FileRef<T>: `(**self).fmt(f)`).
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
struct Data {
    n: i32,
    v: Vec<u8>,
}

#[derive(Debug)]
enum Kind {
    A(u8),
    B { x: i32 },
}

enum FileRef<T: ?Sized + 'static> {
    Static(&'static T),
    Pinned(Rc<T>),
}

impl<T: ?Sized + 'static> Deref for FileRef<T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self {
            FileRef::Static(v) => v,
            FileRef::Pinned(r) => r,
        }
    }
}

impl<T: ?Sized + fmt::Debug + 'static> fmt::Debug for FileRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

struct W(Kind);
impl fmt::Debug for W {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

mod free {
    use std::fmt::Debug;
    pub struct Show<'a>(pub &'a super::Data);
    impl std::fmt::Display for Show<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            (&self.0).fmt(f)
        }
    }
}

fn main() {
    let r: FileRef<Data> = FileRef::Pinned(Rc::new(Data { n: 3, v: vec![1, 2] }));
    println!("{r:?}");
    println!("{r:#?}");
    let s: FileRef<str> = FileRef::Static("hi");
    println!("{s:?}");
    println!("{:?} {:?}", W(Kind::A(7)), W(Kind::B { x: -2 }));
    println!("[{:>4}]", free::Show(&Data::default()));
}
