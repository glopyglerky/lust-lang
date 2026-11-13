// A closure coerced to a fn pointer may return its input's borrow in the
// output slots the pointer type gives its one input lifetime, also when
// other slots are `'static` or come from a type parameter.
use std::sync::OnceLock;

pub struct FileVersion {
    map: OnceLock<Vec<u8>>,
}

pub enum FileRef<T: ?Sized + 'static> {
    Static(&'static T),
    Pinned { key: usize, get: fn(&FileVersion, usize) -> &T },
}

impl<T: ?Sized + 'static> FileRef<T> {
    fn with<'a>(&self, v: &'a FileVersion) -> &'a T
    where
        T: 'a,
    {
        match self {
            FileRef::Static(t) => t,
            FileRef::Pinned { key, get } => get(v, *key),
        }
    }
}

fn decl_map() -> FileRef<Vec<u8>> {
    FileRef::Pinned { key: 0, get: |version, _| version.map.get().expect("made") }
}

#[derive(Debug)]
enum L<'a> {
    B(&'a [u8]),
}

fn take<T>(f: fn(&u8) -> (T, &u8), t: T) -> fn(&u8) -> (T, &u8) {
    let _ = t;
    f
}

static G: u8 = 9;

fn main() {
    let v = FileVersion { map: OnceLock::new() };
    let _ = v.map.set(vec![1, 2]);
    println!("{:?}", decl_map().with(&v));
    let mixed: fn(&u8, usize) -> (&u8, &'static u8) = |a, _| (a, &G);
    let y = 3u8;
    println!("{:?}", mixed(&y, 0));
    let named: for<'a> fn(&'a Vec<u8>) -> Option<L<'a>> = |d| Some(L::B(d.as_slice()));
    println!("{:?}", named(&vec![4, 5]));
    let f = take(|a| (7u8, a), 1u8);
    println!("{:?}", f(&y));
}
