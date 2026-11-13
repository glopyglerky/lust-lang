// expect: L0630
// Valid Rust. A pattern matched through a `&mut` binds by `&mut`; lust's const
// evaluator would write to a copy, so the form is outside the evaluator.
const fn bump(o: &mut Option<u32>) {
    if let Some(x) = o {
        *x += 1;
    }
}
const C: u32 = {
    let mut o = Some(1u32);
    bump(&mut o);
    match o {
        Some(v) => v,
        None => 0,
    }
};
fn main() {
    println!("{}", C);
}
