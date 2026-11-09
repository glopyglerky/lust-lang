// rustc-error: E0282
// lust-expect: L0610
// ARCH 2.12: collect gets its target from the expected type or a
// turbofish. With neither, the receiver of .len() has no known
// head. Pass side: pass/vec_collect_shape.rs.
fn main() {
    let v = (0..3).collect();
    println!("{}", v.len());
}
