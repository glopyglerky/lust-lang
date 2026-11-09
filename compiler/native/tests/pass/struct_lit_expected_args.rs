// A generic literal under an annotation of the same type takes its args for the
// fields, so a closure coerces to `fn(&V) -> &[u32]` and a Box to `Box<[u8]>`.
struct Ver {
    links: Box<[u32]>,
}
struct P<T: ?Sized + 'static> {
    get: fn(&Ver) -> &T,
}
enum FileRef<T: ?Sized + 'static> {
    Static(&'static T),
    Pinned { key: usize, get: fn(&Ver, usize) -> &T },
}
struct W<T: ?Sized> {
    v: Box<T>,
}
static A: [u32; 2] = [4, 5];
fn main() {
    let v = Ver { links: vec![1, 2, 3].into_boxed_slice() };
    let p: P<[u32]> = P { get: |ver| &ver.links };
    let r: FileRef<[u32]> = FileRef::Pinned { key: 1, get: |ver, k| &ver.links[k..] };
    let s: FileRef<[u32]> = FileRef::Static(&A);
    let n = match r {
        FileRef::Pinned { key, get } => get(&v, key).len(),
        FileRef::Static(x) => x.len(),
    };
    let m = match s {
        FileRef::Static(x) => x.len(),
        _ => 0,
    };
    let w: W<[u8]> = W { v: Box::new([1u8, 2]) };
    println!("{} {} {} {}", (p.get)(&v).len(), n, m, w.v.len());
}
