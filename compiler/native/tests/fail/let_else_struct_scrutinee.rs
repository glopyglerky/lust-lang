// expect: L0001
struct S { a: i32 }
fn main() {
    let S { a } = S { a: 1 } else { return; };
    println!("{}", a);
}
