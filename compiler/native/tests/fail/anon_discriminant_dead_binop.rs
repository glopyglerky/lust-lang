// expect: L0113
// rustc E0369: an enum discriminant is checked as a body of its own (Sol review 3, L3).
enum Kind {
    A = {
        if false {
            true + 1;
        }
        0
    },
}
fn main() {
    println!("{}", Kind::A as isize);
}
