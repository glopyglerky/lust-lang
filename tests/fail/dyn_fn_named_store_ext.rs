// expect: L0733
// rustc: lifetime may not live long enough. The `&str` input of `dyn
// FnMut(&mut St<'s>, &str)` is the object's own (late-bound), so the
// closure may not keep it in `St<'s>` (R11).
struct St<'s> {
    name: &'s str,
}
fn run<'s>(st: &mut St<'s>, l: &mut dyn FnMut(&mut St<'s>, &str)) {
    l(st, "ext")
}
fn main() {
    let name = String::from("root");
    let mut st = St { name: &name };
    run(&mut st, &mut |s: &mut St<'_>, e: &str| s.name = e);
    println!("{}", st.name);
}
