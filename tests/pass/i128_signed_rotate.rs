fn o(x: i128) -> i128 { x }
fn u(x: u128) -> u128 { x }
fn main() {
    println!("{}", o(-2).rotate_left(1));
    println!("{}", o(-2).rotate_right(1));
    println!("{}", o(i128::MIN).rotate_left(1));
    for n in [0u32, 63, 64, 65, 127] {
        println!("{} {} {}", o(-2).rotate_left(n), o(-2).rotate_right(n), u(3).rotate_right(n));
    }
}
