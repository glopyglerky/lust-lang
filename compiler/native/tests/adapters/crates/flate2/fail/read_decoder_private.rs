// expect: L0510
// rustc E0412: upstream has no `read::Decoder` type.
fn f(_: &flate2::read::Decoder<&[u8]>) {}

fn main() {
    println!("{}", std::mem::size_of_val(&f));
}
