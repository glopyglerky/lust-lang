// expect: L0001
// rustc: expected identifier, found `1`. cfg-disabled code must still parse, and a
// numeric struct field has no shorthand form (`S { 1 }`), as a pattern or a literal.
fn main() {
    let v = {
        #[cfg(any())]
        Vec { 1 }
        #[cfg(all())]
        { 2 }
    };
    println!("{v}");
}
