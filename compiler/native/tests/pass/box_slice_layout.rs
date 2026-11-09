// Box<[T]> is a fat pointer with the pointer niche.
fn main() {
    let o: Option<Box<[u8]>> = None;
    println!("{} {}", std::mem::size_of::<Option<Box<[u8]>>>(), o.is_none());
    let r: Result<Box<[u8]>, u8> = Ok(vec![1].into_boxed_slice());
    println!("{} {:?}", std::mem::size_of::<Result<Box<[u8]>, u8>>(), r);
    let s: (Box<[u16]>, u8) = (vec![9u16].into_boxed_slice(), 1);
    println!("{} {:?}", std::mem::size_of::<(Box<[u16]>, u8)>(), s);
}
