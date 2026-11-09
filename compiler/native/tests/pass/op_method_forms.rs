// Operator trait methods in method form: scalars in concrete code
// and through bounds, a user impl, assignments, and `String::add`.
use std::ops::{Add, AddAssign, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};

struct M(u8);

impl Add for M {
    type Output = M;
    fn add(self, rhs: M) -> M {
        M(self.0 + rhs.0 + 1)
    }
}

fn via<T: Add<Output = T>>(a: T, b: T) -> T {
    a.add(b)
}

fn bump<T: AddAssign>(a: &mut T, b: T) {
    a.add_assign(b);
}

fn main() {
    println!("{} {} {} {} {}", 20u8.add(3u8), 20u8.sub(3u8), 20u8.mul(3u8), 20u8.div(3u8), 20u8.rem(3u8));
    println!("{} {} {} {} {}", 0b1100u8.bitand(0b1010), 0b1100u8.bitor(0b1010), 0b1100u8.bitxor(0b1010), 1u8.shl(3u32), 32u8.shr(2u32));
    println!("{} {} {}", (-5i32).neg(), 5u8.not(), true.not());
    println!("{} {}", 1.5f64.add(2.25), 1.5f64.neg());
    println!("{} {}", via(20u8, 3u8), via(M(1), M(2)).0);
    println!("{}", 1u8.shl(3));
    let mut a = 1u8;
    a.add_assign(2u8);
    bump(&mut a, 10u8);
    println!("{}", a);
    let s = String::from("a").add("b");
    println!("{}", s);
}
