// expect: L0110
// rustc E0308: break values in a `let` without a type coerce to the first
// break's type, so two closures do not join as a fn pointer.
fn main() {
    let c = || 1;
    let d = || 2;
    let v = loop {
        if true {
            break c;
        }
        break d;
    };
    println!("{}", v());
}
