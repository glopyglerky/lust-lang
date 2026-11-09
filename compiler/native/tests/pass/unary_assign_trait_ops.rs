// Unary and compound-assignment operators through their traits:
// in templates (stubs) and in instances (the impl or the scalar
// operation), including a user `AddAssign` impl.
use std::ops::{AddAssign, Neg};

struct M(u8);

impl AddAssign for M {
    fn add_assign(&mut self, rhs: M) {
        self.0 += rhs.0 + 10;
    }
}

fn neg<T: Neg<Output = T>>(a: T) -> T {
    -a
}

fn add_assign<T: AddAssign + Copy>(mut a: T, b: T) -> T {
    a += b;
    a
}

fn bump<T: AddAssign>(a: &mut T, b: T) {
    *a += b;
}

fn main() {
    println!("{}", neg(5i32));
    println!("{}", add_assign(1u8, 2u8));
    let mut x = M(1);
    bump(&mut x, M(2));
    println!("{}", x.0);
}
