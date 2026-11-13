// Closure operators with both operand types open take the types a later
// call gives them.
fn main() {
    let add = |a, b| a + b;
    let r = add(4294967295u64, 1);
    let lt = |a, b| a < b;
    println!("{} {}", r, lt(1.5, 2.0));
}
