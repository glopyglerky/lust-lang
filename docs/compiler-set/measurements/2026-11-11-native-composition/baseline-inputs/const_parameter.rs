fn first<const N: usize>(values: [i32; N]) -> i32 { values[0] }
unsafe fn add(value: i32) -> i32 { value + 1 }
fn main() { println!("{}", unsafe { add(first([41, 9, 2])) }); }
