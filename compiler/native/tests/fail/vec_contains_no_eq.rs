// expect: L0112
struct NoEq {
    n: i32,
}
fn main() {
    let v = vec![NoEq { n: 1 }];
    println!("{}", v.contains(&NoEq { n: 1 }));
}
