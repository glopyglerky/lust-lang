// expect: L0612
// rustc E0276: a where clause over concrete types that does not hold.
pub trait Val {
    fn val(&self) -> u32;
}

impl<T: Copy> Val for T {
    fn val(&self) -> u32
    where
        String: Copy,
    {
        7
    }
}

fn main() {
    println!("{}", 5i32.val());
}
