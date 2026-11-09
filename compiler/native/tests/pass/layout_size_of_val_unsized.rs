// `size_of_val` through a `T: ?Sized` parameter: the template check
// holds a placeholder and each instance answers at runtime. A slice
// is its length times the element size, a `str` is its byte length.
fn f<T: ?Sized>(t: &T) -> usize {
    std::mem::size_of_val(t)
}

fn main() {
    println!("{}", f(&[1u8, 2, 3] as &[u8]));
    println!("{} {} {}", f("hello"), f(&[1u16, 2, 3] as &[u16]), f(&42u64));
    let a = [1u32, 2, 3, 4];
    let s: &[u32] = &a;
    println!("{} {}", std::mem::size_of_val(s), std::mem::size_of_val("abc"));
}
