// expect: L0110
// rustc E0091: type parameter `T` is never used in the alias.
type Same<T> = i32;

fn main() {
    let x: Same<u8> = 3;
    println!("{}", x);
}
