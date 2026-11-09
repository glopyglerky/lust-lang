// Shapes the template check (each generic body checked once against
// its bounds, with rigid type params) must accept: trait calls on
// params and references to params, default bodies calling the trait
// and `Self::f()`, Fn bounds forwarded to another generic fn, tuple
// fields of generic structs, `?Sized + Copy` (Copy implies Sized),
// and `size_of::<T>()` at run time.
trait Shape {
    fn area(&self) -> u32;
    fn scale(&mut self, k: u32);
    fn unit() -> Self;
    fn twice_area(&self) -> u32 {
        self.area() * 2
    }
    fn unit_area() -> u32
    where
        Self: Sized,
    {
        Self::unit().area()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Sq(u32);

impl Shape for Sq {
    fn area(&self) -> u32 {
        self.0 * self.0
    }
    fn scale(&mut self, k: u32) {
        self.0 *= k;
    }
    fn unit() -> Self {
        Sq(1)
    }
}

fn total<T: Shape>(a: &T, b: &T) -> u32 {
    a.area() + b.twice_area()
}

fn grow<T: Shape + Clone + std::fmt::Debug + PartialEq>(mut t: T, k: u32) -> T {
    let before = t.clone();
    t.scale(k);
    println!("{:?} {}", before, before == t);
    t
}

fn call_fn<F: Fn(u8) -> u8>(f: F) -> u8 {
    f(1) + f(2)
}

fn pass_on<F: Fn(u8) -> u8>(f: F) -> u8 {
    call_fn(f)
}

fn call_mut<F: FnMut() -> u8>(mut f: F) -> u8 {
    f() + f()
}

fn call_once<F: FnOnce(u8) -> u16>(f: F) -> u16 {
    f(7)
}

struct P<A, B>(A, B);

fn second<A, B>(p: P<A, B>) -> B {
    p.1
}

fn swap<A, B>(p: P<A, B>) -> P<B, A> {
    P(p.1, p.0)
}

fn copy_out<T: ?Sized + Copy>(x: &T) -> T {
    *x
}

fn size<T>() -> usize {
    std::mem::size_of::<T>()
}

fn main() {
    println!("{}", total(&Sq(2), &Sq(3)));
    println!("{}", grow(Sq(2), 3).area());
    println!("{}", Sq::unit_area());
    println!("{}", pass_on(|x| x * 3));
    println!("{}", call_mut(|| 4));
    println!("{}", call_once(|x| x as u16 * 300));
    println!("{}", second(P(1u8, "two")));
    let s = swap(P(1u8, 'c'));
    println!("{} {}", s.0, s.1);
    println!("{}", copy_out(&5u8));
    println!("{} {}", size::<u32>(), size::<(u8, u64)>());
}
