// capability: rejected: destructuring in `let` without an initializer
// expect: L9001

fn main() {
    let (a, b);
    a = 1;
    b = 2;
    println!("{}", a + b);
}
