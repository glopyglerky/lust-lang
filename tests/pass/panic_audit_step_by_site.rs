// `step_by(0)` asserts inside core's StepBy::new.
// panic: assertion failed: step != 0
fn main() {
    let n = std::env::args().count();
    let v = vec![1, 2, 3];
    println!("{}", v.iter().step_by(n).count());
    println!("{}", v.iter().step_by(n - 1).count());
}
