// expect: L0634
// rustc E0435: an array length in a body's written type cannot name a local, also in a dead branch.
fn main() {
    let x = 1usize;
    let a: [u8; {
        if false {
            x;
        }
        2
    }] = [0; 2];
    println!("{}", a.len());
}
