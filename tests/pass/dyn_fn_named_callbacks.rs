// ts-rust callback types (R11): named lifetimes in `dyn FnMut` sugar, with an
// object lifetime of their own.
struct State<'s> { name: &'s str, hits: u32 }
type Loader<'s> = dyn FnMut(&mut State<'s>, u32, &str) -> Option<usize>;
type Callback<'f, 's> = &'f mut dyn FnMut(&mut State<'s>, u32) -> u32;
type Referenced<'c, 'a> = dyn FnMut(&mut State<'a>, &str) -> (bool, bool) + 'c;
fn run<'s>(st: &mut State<'s>, l: &mut Loader<'s>) -> Option<usize> { l(st, 2, "ext") }
fn cb<'f, 's>(st: &mut State<'s>, c: Callback<'f, 's>) -> u32 { c(st, 5) }
fn each<'a>(st: &mut State<'a>, f: &mut Referenced<'_, 'a>) -> bool { f(st, "p").0 }
fn main() {
    let name = String::from("root");
    let mut st = State { name: &name, hits: 0 };
    let r = run(&mut st, &mut |s: &mut State<'_>, k, e: &str| { s.hits += k; Some(s.name.len() + e.len()) });
    let mut extra = 1;
    let c = cb(&mut st, &mut |s, k| { extra += k; s.hits + k });
    let b = each(&mut st, &mut |s, p| (s.name.len() > p.len(), true));
    println!("{:?} {} {} {} {}", r, c, b, extra, st.hits);
}
