// expect: L0110
// rustc E0277: `W<T>` is unsized when `T: ?Sized`, so `size_of` rejects.
struct W<T> {
    b: T,
}

fn f<T: ?Sized>() -> usize {
    std::mem::size_of::<W<T>>()
}

fn main() {
    println!("{}", f::<u8>());
}
