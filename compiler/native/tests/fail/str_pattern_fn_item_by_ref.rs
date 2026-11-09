// expect: L0612
// rustc E0631: a str pattern fn takes a `char`, not a `&char`.
fn space(c: &char) -> bool {
    *c == ' '
}
fn main() {
    println!("{}", "a b".trim_matches(space));
}
