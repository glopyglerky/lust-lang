// feature: closures.fn_ptr_lifetime
// form: `&'static` inside `Option` and `'_` Adt args in fn pointer aliases, as in ts-rust binder/reference_resolver.rs and effect/rule.rs
struct Checker { n: u32 }
struct Message { t: &'static str }
static M: Message = Message { t: "hi" };
struct RuleContext<'a, 'b> { a: &'a str, b: &'b mut Vec<u32> }
type Resolve = fn(&mut Checker, &str, Option<&'static Message>, bool) -> u32;
pub type Run = fn(&mut RuleContext<'_, '_>) -> Vec<u32>;
fn resolve(c: &mut Checker, s: &str, m: Option<&'static Message>, b: bool) -> u32 {
    c.n += 1;
    if b { s.len() as u32 + m.map(|m| m.t.len() as u32).unwrap_or(0) } else { c.n }
}
fn run(c: &mut RuleContext<'_, '_>) -> Vec<u32> { c.b.push(c.a.len() as u32); c.b.clone() }
fn main() {
    let f: Resolve = resolve;
    let r: Run = run;
    let mut c = Checker { n: 1 };
    let mut v = Vec::new();
    let s = String::from("xy");
    let mut ctx = RuleContext { a: &s, b: &mut v };
    println!("{} {:?}", f(&mut c, "abc", Some(&M), true), r(&mut ctx));
}
