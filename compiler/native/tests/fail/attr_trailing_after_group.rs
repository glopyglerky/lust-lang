// expect: L0001
// rustc: expected `]`, found `x`. An attribute input is one group; nothing may follow it.
#[derive(Debug)x]
struct S;

fn main() {
    println!("{:?}", S);
}
