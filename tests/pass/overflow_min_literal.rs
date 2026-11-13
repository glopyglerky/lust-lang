// A minimum-value literal under a minus is a magnitude, not a
// negation overflow.
fn main() {
    let a: i128 = -170141183460469231731687303715884105728i128;
    println!("{}", a + 1);
    let b: i8 = -128i8;
    println!("{}", b + 1);
}
