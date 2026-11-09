// expect: L0112
// rustc accepts this (Vec<Range<i32>> is Debug); lust's Debug glue has
// no range form, so lust rejects it.
fn main() {
    println!("{:?}", vec![1..2]);
}
