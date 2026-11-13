// Trait methods named through their trait as values and calls:
// `Into::into`, `From::from`, `Clone::clone`, `ToString::to_string`,
// `Default::default`, and user traits. Self and the trait inputs
// come from the use around the path, as in rustc.
#[derive(Clone, Debug)]
struct W(u32);

struct S(String);
impl From<&str> for S {
    fn from(s: &str) -> S {
        S(format!("<{s}>"))
    }
}

trait Mk {
    fn mk(x: u8) -> Self;
}
impl Mk for W {
    fn mk(x: u8) -> W {
        W(x as u32 * 10)
    }
}
trait Conv {
    fn conv(&self) -> u32;
}
impl Conv for W {
    fn conv(&self) -> u32 {
        self.0 + 1
    }
}

fn details(d: impl IntoIterator<Item = impl Into<String>>) -> Vec<String> {
    d.into_iter().map(Into::into).collect()
}

fn main() {
    // The ts-rust shape, with array and Vec call sites.
    println!("{:?}", details(["a", "b"]));
    println!("{:?}", details(vec!["c"]));

    let a: Vec<String> = vec!["x"].into_iter().map(Into::into).collect();
    let b: Vec<String> = vec!["y"].into_iter().map(From::from).collect();
    let c: Vec<u32> = vec![1u8, 2].into_iter().map(Into::into).collect();
    let d: Vec<String> = vec![String::from("z")].into_iter().map(Into::into).collect();
    let e: Vec<String> = vec![1, 2].iter().map(ToString::to_string).collect();
    let f: Vec<i32> = vec![3i32].iter().map(Clone::clone).collect();
    let g: Vec<W> = vec![W(4)].iter().map(Clone::clone).collect();
    let h: Vec<W> = vec![1u8, 2].into_iter().map(Mk::mk).collect();
    let i: Vec<u32> = vec![W(9)].iter().map(Conv::conv).collect();
    let j: Vec<S> = vec!["q"].into_iter().map(Into::into).collect();
    let k: Vec<String> = vec!["r"].into_iter().map(std::convert::Into::into).collect();
    println!("{a:?} {b:?} {c:?} {d:?} {e:?} {f:?} {g:?} {:?} {i:?} {:?} {k:?}", h.iter().map(|w| w.0).collect::<Vec<_>>(), j.iter().map(|s| &s.0).collect::<Vec<_>>());

    // Direct calls through the trait name.
    let s1: String = Into::into("a");
    let s2: String = From::from("b");
    let s3: String = ToString::to_string(&3);
    let n: i32 = Clone::clone(&4);
    let s4: String = Default::default();
    println!("{s1} {s2} {s3} {n} {s4:?}");

    // Stored in a local, called later.
    let f = Into::into;
    let s5: String = f("held");
    println!("{s5}");

    // `Default::default` as the value an `unwrap_or_else` calls.
    let v: String = None::<String>.unwrap_or_else(Default::default);
    println!("{v:?}");
}
