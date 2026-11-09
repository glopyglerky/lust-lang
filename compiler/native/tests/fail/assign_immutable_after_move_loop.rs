// expect: L0302
// rustc E0384: the loop body reassigns an immutable local after moving it.
fn d(i: i32) -> String { format!("s{}", i) }
fn main() {
    let a = d(1);
    for _ in 0..2 { drop(a); a = d(2); }
}
