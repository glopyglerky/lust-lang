// Methods of a type parameter's where clauses are inherent candidates:
// they win over a blanket impl of another trait, by receiver type.
trait A { fn f(&self) -> u8; fn make() -> u8; }
trait B { fn f(&self) -> u8; fn make() -> u8; }
impl<T> B for T { fn f(&self) -> u8 { 2 } fn make() -> u8 { 20 } }
impl A for u8 { fn f(&self) -> u8 { 1 } fn make() -> u8 { 10 } }
trait Base { fn base(&self) -> u8; }
trait Sub: Base { fn sub(&self) -> u8 { self.base() + 1 } }
impl Base for u8 { fn base(&self) -> u8 { 30 } }
impl Sub for u8 {}
fn by_ref<T: A>(t: &T) -> u8 { t.f() }
fn by_val<T: A>(t: T) -> u8 { t.f() }
fn path<T: A>() -> u8 { T::make() }
fn sup<T: Sub>(t: &T) -> u8 { t.base() + t.sub() }
fn outer<T: A + Sub>(t: &T) -> u8 { by_ref(t) + sup(t) }
fn main() {
    println!("{} {} {} {}", by_ref(&5u8), by_val(5u8), path::<u8>(), sup(&5u8));
    println!("{}", outer(&5u8));
}
