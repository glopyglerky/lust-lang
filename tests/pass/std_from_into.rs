// std From/Into between builtin types (W-STD-B): `x.into()` with the
// target from the expected type or a generic bound, `impl Into<T>`
// arguments, `T::from(x)` forms, and user From impls through Into in
// generic instances.
use std::borrow::Cow;
use std::rc::Rc;

#[derive(Debug)]
struct JsonError {
    message: String,
}

impl JsonError {
    fn new(message: impl Into<String>) -> JsonError {
        JsonError { message: message.into() }
    }
}

struct Source {
    bytes: Vec<u8>,
}

impl Source {
    fn from_bytes(bytes: impl Into<Vec<u8>>) -> Source {
        let bytes = bytes.into();
        Source { bytes }
    }
}

struct Meters(f64);

impl From<i32> for Meters {
    fn from(x: i32) -> Meters {
        Meters(x as f64)
    }
}

fn total<T: Into<Meters>>(xs: Vec<T>) -> f64 {
    let mut t = 0.0;
    for x in xs {
        t += x.into().0;
    }
    t
}

fn label<'a, S: Into<Cow<'a, str>>>(s: S) -> Cow<'a, str> {
    s.into()
}

fn main() {
    let name = String::from("n");
    let errs = vec![JsonError::new("lit"), JsonError::new(format!("f{}", 1)), JsonError::new(name.clone()), JsonError::new(&name)];
    for e in &errs {
        println!("{:?} {}", e, e.message.capacity() >= e.message.len());
    }
    let a = Source::from_bytes(vec![1u8, 2]);
    let b = Source::from_bytes("ab");
    let c = Source::from_bytes(String::from("cd"));
    let d = Source::from_bytes(&[5u8, 6][..]);
    println!("{:?} {:?} {:?} {:?}", a.bytes, b.bytes, c.bytes, d.bytes);
    println!("{}", total(vec![1, 2, 3]));
    println!("{}", total(vec![Meters(0.5)]));
    let l1 = label("x");
    let l2 = label(String::from("y"));
    println!("{l1} {l2} {}", matches!(l1, Cow::Borrowed(_)));
    let s: String = 'c'.into();
    let t: String = Cow::Borrowed("cow").into();
    let v: Vec<i32> = [3, 4].into();
    let w: Vec<char> = (&['a', 'b'][..]).into();
    let o: Option<u8> = 7.into();
    let bx: Box<i32> = 8.into();
    let rc: Rc<String> = String::from("rc").into();
    println!("{s} {t} {v:?} {w:?} {o:?} {bx} {rc}");
    let bs: Box<[u8]> = vec![1u8, 2].into();
    let back: Vec<u8> = bs.into();
    println!("{back:?} {}", back.capacity());
    let s2 = String::from(Cow::Borrowed("sf"));
    let v2 = Vec::from("vf");
    let v3 = Vec::from(String::from("vs"));
    let c2: Cow<str> = Cow::from("cf");
    let c3 = Cow::from(vec![1, 2]);
    println!("{s2} {v2:?} {v3:?} {c2} {c3:?}");
    let same: String = String::from("same").into();
    println!("{same}");
}
