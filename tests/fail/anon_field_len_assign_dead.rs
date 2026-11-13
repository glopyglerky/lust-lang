// expect: L0302
// rustc E0384: an array length in a field type is checked as a body of its own (Sol review 3, L2).
struct Packet {
    bytes: [u8; {
        if false {
            let n = 1;
            n = 2;
        }
        1
    }],
}
fn main() {
    println!("{}", std::mem::size_of::<Packet>());
}
