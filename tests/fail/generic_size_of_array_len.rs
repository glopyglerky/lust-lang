// expect: L0110
// rustc: an array length cannot depend on a generic parameter.
fn f<T>() -> usize {
    let a = [0u8; std::mem::size_of::<T>()];
    a.len()
}

fn main() {
    println!("{}", f::<u32>());
}
