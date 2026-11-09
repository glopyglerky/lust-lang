// expect: L0523
// rustc E0658: `cfg(overflow_checks)` is unstable.
#[cfg(overflow_checks)]
fn which() -> i32 {
    1
}

#[cfg(not(overflow_checks))]
fn which() -> i32 {
    2
}

fn main() {
    println!("{}", which());
}
