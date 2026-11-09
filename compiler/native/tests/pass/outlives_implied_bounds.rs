// Implied outlives bounds rustc accepts: from parameter and return
// types, struct params (inferred outlives), impl headers, `T: 'static`,
// and declared `'b: 'a` facts.
use std::marker::PhantomData;
trait Tr<X> {}
impl<X> Tr<X> for () {}
trait Obj {
    fn v(&self) -> u8;
}
impl Obj for u8 {
    fn v(&self) -> u8 {
        *self
    }
}
struct W<'a, T> {
    r: &'a T,
}
struct S<'a, T>(&'a T)
where
    (): Tr<&'a T>;
struct Own<T>(T);
trait A {
    type Out;
}
impl<'a, T> A for Own<&'a T> {
    type Out = &'a T;
}
impl<'a, T> W<'a, T> {
    fn get(&self) -> &'a T
    where
        (): Tr<&'a T>,
    {
        self.r
    }
}
fn first<'a, T: Copy, I: IntoIterator<Item = &'a T>>(_s: &'a [T], i: I) -> Option<T> {
    i.into_iter().next().copied()
}
fn by_ret<'a, T>(v: &'a [T]) -> Option<&'a T>
where
    (): Tr<&'a T>,
{
    v.first()
}
fn by_struct<'a, T: Copy>(w: W<'a, T>) -> T
where
    (): Tr<&'a T>,
{
    *w.r
}
fn by_static<'a, T: 'static>(_z: &'a ())
where
    (): Tr<&'a T>,
{
}
fn by_fact<'a, 'b: 'a>(x: &'a (), _y: &'b ()) -> usize
where
    (): Tr<&'a &'b ()>,
{
    let p: PhantomData<&'a &'b ()> = PhantomData;
    let _ = p;
    let _ = x;
    1
}
fn boxed<'a, T: Obj>(x: &'a T) -> Box<dyn Obj + 'a> {
    Box::new(x.v())
}
fn boxed_param<'a, T: Obj>(x: T, _y: &'a T) -> Box<dyn Obj + 'a> {
    Box::new(x)
}
fn boxed_static<'a, T: Obj + 'static>(x: T, _z: &'a ()) -> Box<dyn Obj + 'a> {
    Box::new(x)
}
fn main() {
    let v = vec![4u8, 5];
    println!("{:?}", first(&v, &v));
    println!("{:?}", by_ret(&v));
    println!("{}", by_struct(W { r: &7u8 }));
    by_static::<u8>(&());
    println!("{}", by_fact(&(), &()));
    let w = W { r: &9u8 };
    println!("{}", w.get());
    let s = S(&1u8);
    println!("{}", s.0);
    let o: <Own<&u8> as A>::Out = &3;
    println!("{}", o);
    println!("{} {} {}", boxed(&2u8).v(), boxed_param(6u8, &6).v(), boxed_static(8u8, &()).v());
}
