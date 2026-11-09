// expect: L0312
// rustc E0506: `let _ = r;` mentions `r`, so its loan is live at the assignment.
fn main() {
    let mut a = 1;
    let r = &mut a;
    a = 2;
    let _ = r;
    println!("{}", a);
}
