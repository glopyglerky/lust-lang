// expect: L0301
// rustc E0382: an array repeat count is an anonymous const.
struct S {
    x: usize,
}
fn main() {
    let a = [0u8; {
        let s = S { x: 1 };
        let t = s;
        s.x + t.x
    }];
    println!("{}", a.len());
}
