// capability: rejected: tuple struct patterns with field numbers
// expect: L0820

struct P(u32, u32);

fn main() {
    let P { 0: a, 1: b } = P(1, 2);
    println!("{}", a + b);
}
