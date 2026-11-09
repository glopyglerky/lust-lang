// expect: L0634
// rustc E0435: a repeat count cannot name a local (lust's borrowck crashed on it).
fn main() {
    let x = 1usize;
    let a = [0u8; {
        if false {
            x;
        }
        2
    }];
    println!("{}", a.len());
}
