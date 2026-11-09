// expect: L0102
// rustc E0433/E0412: HashMap is not in the prelude.
fn main() {
    let m: HashMap<u32, u32> = HashMap::new();
    println!("{}", m.len());
}
