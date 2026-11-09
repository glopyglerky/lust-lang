// expect: L0110
// rustc E0277: `size_of` needs `T: Sized`; a `?Sized` bound is not enough.
fn f<T: ?Sized>() -> usize {
    std::mem::size_of::<T>()
}

fn main() {
    println!("{}", f::<u8>());
}
