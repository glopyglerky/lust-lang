// expect: L0510
// rustc E0451: a struct literal cannot set private fields of a std type.
use std::time::Duration;
fn main() {
    let d = Duration { secs: 1, nanos: 0 };
    println!("{:?}", d);
}
