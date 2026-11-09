// W-BORROW3: a plain type alias of a fn pointer type keeps its `&'static`
// references, also inside `Option` (ts-rust binder/reference_resolver.rs,
// declarations/diagnostics.rs).
struct Checker { n: u32 }
struct Message { t: &'static str }
static M: Message = Message { t: "hi" };
type Resolve = fn(&mut Checker, u32, &str, Option<&'static Message>, bool) -> u32;
type Selector = fn(u32, &Checker) -> Option<&'static Message>;
type Plain = fn(&str, &'static Message) -> usize;
fn resolve(c: &mut Checker, x: u32, s: &str, m: Option<&'static Message>, b: bool) -> u32 {
    c.n += x;
    if b { s.len() as u32 + m.map(|m| m.t.len() as u32).unwrap_or(0) } else { 0 }
}
fn select(n: u32, c: &Checker) -> Option<&'static Message> { if n + c.n > 0 { Some(&M) } else { None } }
fn plain(s: &str, m: &'static Message) -> usize { s.len() + m.t.len() }
fn main() {
    let f: Resolve = resolve;
    let table: [Selector; 1] = [select];
    let g: Plain = plain;
    let mut c = Checker { n: 1 };
    let local = String::from("abc");
    println!("{}", f(&mut c, 2, &local, Some(&M), true) + c.n);
    let kept;
    {
        let c2 = Checker { n: 0 };
        kept = table[0](1, &c2);
    }
    println!("{} {}", kept.unwrap().t, g(&local, &M));
}
