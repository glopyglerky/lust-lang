// `impl Into<String>` and kin: std's builtin From impls (String from
// &str, &String, char; Box<T> and Rc<T> from T; From<T> for T) through
// core's blanket Into, in instances and in plain `.into()` calls.
use std::rc::Rc;
#[derive(Debug)]
struct Dirs { n: u32 }
#[derive(Debug)]
struct Name(String);
impl From<&str> for Name { fn from(s: &str) -> Name { Name(s.to_string()) } }
impl From<String> for Name { fn from(s: String) -> Name { Name(s) } }
fn take(s: impl Into<String>) -> usize { s.into().len() }
fn own(s: impl Into<String>) -> String { let mut t: String = s.into(); t.push('!'); t }
fn fwd(s: impl Into<String>) -> String { inner(s) }
fn inner<S: Into<String>>(s: S) -> String { s.into() }
fn dirs(d: impl Into<Rc<Dirs>>) -> Rc<Dirs> { d.into() }
fn boxed(x: impl Into<Box<Vec<u8>>>) -> usize { x.into().len() }
fn name(n: impl Into<Name>) -> Name { n.into() }
fn both(a: impl Into<String>, b: impl Into<String>) -> String { let (a, b): (String, String) = (a.into(), b.into()); a + &b }
fn main() {
    let s = String::from("hello");
    println!("{} {} {} {}", take("abcd"), take(s.clone()), take(&s), take('x'));
    println!("{} {} {}", own("a"), own(format!("{}{}", 1, 2)), own(&s));
    println!("{} {}", fwd("f"), fwd(String::from("g")));
    let d = dirs(Dirs { n: 3 }); let d2 = dirs(d.clone()); println!("{} {} {:?}", d.n, Rc::strong_count(&d2), d2);
    println!("{} {}", boxed(vec![1u8, 2]), boxed(Box::new(vec![3u8])));
    println!("{:?} {:?}", name("n1"), name(String::from("n2")));
    println!("{}", both("x", String::from("y")));
    let a: String = "lit".into(); let b: String = 'c'.into(); let c: String = (&a).into(); let v: Vec<i32> = vec![1].into();
    let bx: Box<String> = a.clone().into(); let r: Rc<u8> = 7u8.into();
    println!("{} {} {} {:?} {} {}", a, b, c, v, bx, r);
}
