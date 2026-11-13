// panic: attempt to add with overflow
// next_power_of_two overflows at core's uint_impl! line for its type (R13).
fn main() {
    let n = std::env::args().count();
    let x = 64u8 + n as u8;
    println!("{}", x.next_power_of_two());
    let y = 128u8 + n as u8;
    println!("{}", y.next_power_of_two());
}
