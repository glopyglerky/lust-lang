// A bound puts its trait's methods in scope for its own type param, with no
// `use`: by value, through `&`, `Box`, a supertrait, an impl's bounds, a trait
// default body, an `impl Trait` arg, and by path (`T::k()`). In a trait impl
// the impl's own trait is in scope too (rustc's current-trait rule).
mod m {
    pub trait Tr {
        fn f(&self) -> i32;
        fn k() -> i32;
    }
    impl Tr for i32 {
        fn f(&self) -> i32 {
            *self + 1
        }
        fn k() -> i32 {
            40
        }
    }
}
trait Num: std::ops::Add<Output = Self> + Copy {}
impl Num for u8 {}
fn by_value<T: std::ops::Add<Output = T>>(x: T, y: T) -> T {
    x.add(y)
}
fn by_ref<T: std::ops::Add<Output = T> + Copy>(x: &T, y: T) -> T {
    x.add(y)
}
fn by_box<T: std::ops::Neg<Output = T>>(x: Box<T>) -> T {
    x.neg()
}
fn by_super<T: Num>(x: T) -> T {
    x.add(x)
}
fn user_path<T: m::Tr>() -> i32 {
    T::k()
}
fn user<T: m::Tr>(x: T) -> i32 {
    x.f()
}
fn arg(x: impl m::Tr, h: &mut impl std::hash::Hasher) -> i32 {
    h.write_u8(1);
    x.f()
}
fn hashed<T: std::hash::Hash>(x: &T) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    x.hash(&mut h);
    use std::hash::Hasher;
    h.finish()
}
struct W<T>(T);
impl<T: std::ops::Add<Output = T> + Copy> W<T> {
    fn twice(&self) -> T {
        self.0.add(self.0)
    }
}
trait Dbl: std::ops::Add<Output = Self> + Copy {
    fn dbl(self) -> Self {
        self.add(self)
    }
}
impl Dbl for i64 {}
struct H(u64);
impl std::hash::Hasher for H {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, b: &[u8]) {
        for x in b {
            self.0 = self.0 * 31 + *x as u64;
        }
    }
    fn write_u16(&mut self, i: u16) {
        self.write_u8(i as u8);
    }
}
fn main() {
    let mut d = std::collections::hash_map::DefaultHasher::new();
    let mut h = H(0);
    std::hash::Hasher::write_u16(&mut h, 7);
    println!("{} {} {} {}", by_value(1u8, 2), by_ref(&3i32, 4), by_box(Box::new(5i16)), by_super(6u8));
    println!("{} {} {} {}", user_path::<i32>(), user(7), arg(8, &mut d), W(9u64).twice());
    println!("{} {} {}", 10i64.dbl(), h.0, hashed(&5u8) == hashed(&5u8));
}
