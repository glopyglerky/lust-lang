// expect: L0610
// rustc E0282: nothing fixes the collection type partition builds.
fn main() {
    let (a, b) = (0..4u32).partition(|x| x % 2 == 0);
    println!("{}", a.len() + b.len());
}
