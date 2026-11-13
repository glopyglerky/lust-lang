// expect: L0113
// `Vec::new` as a value (an eta closure, parser.rs `eta_closure`) under
// `+` reports in check (it reached lowering as an unresolved place
// before, L0310).
fn main() {
    let mut v = Vec::new+();
    println!("{}", v[0]);
}
