// `Vec::with_capacity` past isize::MAX bytes: RawVec's capacity overflow.
// panic: capacity overflow
fn main() {
    let n = std::env::args().count();
    print!("partial");
    let v: Vec<u32> = Vec::with_capacity(usize::MAX / 4 + n);
    println!("{}", v.len());
}
