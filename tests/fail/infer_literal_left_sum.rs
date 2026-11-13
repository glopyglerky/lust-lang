// expect: L0610
// rustc E0284: `{integer}: Add<?S>` does not fix the `sum` result `?S`;
// the left literal gives the right operand no type (hunt6 class).
fn main() {
    let x = 3 + [1, 1].iter().sum();
    println!("{}", x);
}
