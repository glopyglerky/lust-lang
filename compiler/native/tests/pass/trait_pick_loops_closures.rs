// Trait picks inside loops, closures, and nested generic calls.
struct S(u8);
impl S { fn val(&self) -> u8 { 100 } }
trait V { fn val(&self) -> u8; }
impl V for S { fn val(&self) -> u8 { self.0 } }
impl V for u8 { fn val(&self) -> u8 { *self + 1 } }
fn sum<T: V>(xs: &[T]) -> u32 { let mut s = 0u32; for x in xs { s += x.val() as u32; } s }
fn apply<T: V, F: Fn(&T) -> u8>(t: &T, f: F) -> u8 { f(t) + t.val() }
fn outer<T: V>(t: &T) -> u8 { apply(t, |x: &T| x.val()) }
fn main() {
    println!("{} {}", sum(&[S(1), S(2)]), sum(&[1u8, 2u8]));
    println!("{} {}", outer(&S(3)), outer(&4u8));
}
