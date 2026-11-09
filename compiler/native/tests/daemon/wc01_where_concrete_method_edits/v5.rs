// A where clause over concrete types on a blanket impl method, edited
// on one warm daemon: added (holds), a scalar From bound, one that does
// not hold (an error), then removed again.
pub trait Val {
    fn val(&self) -> u32;
}

impl<T: Copy> Val for T {
    fn val(&self) -> u32 {
        7
    }
}

fn main() {
    println!("{}", 5i32.val());
}
