// expect: L0631
// rustc E0493: a `String` local in a dead arm of an inline const drops.
fn main() {
    let v = const {
        match true {
            true => 1,
            false => {
                let s = String::new();
                s.len()
            }
        }
    };
    println!("{}", v);
}
