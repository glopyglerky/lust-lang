// `x.into()` on an unsized place (`v[1..]`, `*s`, `*boxed`, a field of
// slice type) autorefs: Self is `&[T]`/`&str`, as in rustc's probe
// (core's blanket `Into` needs `Self: Sized`). Also the autoref of a
// range-indexed receiver keeps its loan on the base, not on a temporary.
use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug)]
struct My(usize, u32);
impl From<&[u32]> for My {
    fn from(s: &[u32]) -> My {
        My(s.len(), s.iter().sum())
    }
}
impl From<&str> for My {
    fn from(s: &str) -> My {
        My(s.len(), 0)
    }
}

struct W {
    b: Box<[u32]>,
    s: Box<str>,
}

fn take(x: Arc<str>) -> usize {
    x.len()
}

fn tail<T: Clone>(v: &Vec<T>) -> Rc<[T]> {
    v[1..].into()
}

fn main() {
    let v = vec![1u32, 2, 3];
    let s = String::from("hello");
    let st: &str = "world";
    let w = W { b: vec![9u32, 10].into_boxed_slice(), s: Box::from("field") };

    let b: Box<[u32]> = v[1..].into();
    let a: Arc<[u32]> = v[1..].into();
    let r: Rc<[u32]> = v[..2].into();
    let vv: Vec<u32> = (*v).into();
    let c: Cow<[u32]> = v[1..].into();
    let m: My = w.b[..].into();
    let rf: &[u32] = (*w.b).into();
    println!("{:?} {:?} {:?} {:?} {:?} {:?} {:?}", b, a, r, vv, c, m, rf);

    let bs: Box<str> = s[..].into();
    let rs: Rc<str> = (*st).into();
    let owned: String = s[1..].into();
    let cs: Cow<str> = (*w.s).into();
    let ms: My = s[2..].into();
    println!("{} {} {} {} {:?} {}", bs, rs, owned, cs, ms, take(s[1..4].into()));
    println!("{:?} {:?}", tail(&v), Some::<Box<str>>(st[1..].into()));

    let mut v2 = vec![5u32, 4, 3, 2];
    let it = v2[1..].iter();
    println!("{}", it.len());
    v2[1..].sort();
    let first = v2[1..].first_mut().unwrap();
    *first += 10;
    println!("{:?}", v2);
}
