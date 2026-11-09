// The closure stores `&x` into its captured `&mut out`; the `&mut` borrow
// ends with the last call, so `out` is usable after.
fn main() {
    let x = 5;
    let mut out: Vec<&i32> = Vec::new();
    let mut add = || out.push(&x);
    add();
    add();
    println!("{}", out.len());
}
