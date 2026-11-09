// rustc accepts: 'static data through a 'static alias, and the single input
// lifetime ('static here) bound to an elided output.
type S = &'static str;

fn keep(x: S) -> usize {
    x.len()
}

fn same(x: &'static str) -> &str {
    x
}

fn main() {
    println!("{} {}", keep("abc"), same("de"));
}
