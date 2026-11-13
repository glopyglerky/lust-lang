// ARCH 2.10 and fx-tmpl-consts gap 3: a trait function as a value on a
// `<S as Tr>::mk` (also to pick between two traits), and method fns
// (`S::get`), called directly or coerced to fn pointers.
trait A { fn mk() -> u8; }
trait B { fn mk() -> u8; }
trait G { fn get(&self) -> u8; }
struct S(u8);
impl A for S { fn mk() -> u8 { 1 } }
impl B for S { fn mk() -> u8 { 2 } }
impl G for S { fn get(&self) -> u8 { self.0 } }
struct T;
impl B for T { fn mk() -> u8 { 3 } }
fn apply(f: fn() -> u8) -> u8 { f() }
fn main() {
    let f = <S as A>::mk;
    let g = <S as B>::mk;
    let h = T::mk;
    let get = S::get;
    let get2 = <S as G>::get;
    println!("{} {} {} {} {}", f(), g(), h(), get(&S(7)), get2(&S(8)));
    println!("{} {} {}", apply(<S as A>::mk), apply(T::mk), apply(<T as B>::mk));
}
