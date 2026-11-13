// expect: L0634
// rustc E0435: an inline const cannot name a local of the fn around it (lust's borrowck crashed on it).
fn main() {
    let x = 1u32;
    let y = const {
        if false {
            x;
        }
        1
    };
    println!("{}", y);
}
