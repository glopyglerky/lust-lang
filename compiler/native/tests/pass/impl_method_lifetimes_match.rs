// Impl method signatures that use trait and method lifetimes as the trait does.
trait Tr<'a> { fn f(&self, x: &'a u8) -> u8; fn g(&self) -> &'a u8; }
struct S<'a>(&'a u8);
impl<'a> Tr<'a> for S<'a> { fn f(&self, x: &'a u8) -> u8 { *x + *self.0 } fn g(&self) -> &'a u8 { self.0 } }
trait Lt { fn h<'b>(&'b self, x: &'b u8) -> &'b u8; }
impl Lt for u8 { fn h<'c>(&'c self, x: &'c u8) -> &'c u8 { if *x > *self { x } else { self } } }
struct W<'a>(&'a str);
impl<'a> AsRef<str> for W<'a> { fn as_ref(&self) -> &str { self.0 } }
impl<'a> PartialEq<&'a str> for W<'a> { fn eq(&self, o: &&'a str) -> bool { self.0 == *o } }
fn main() { let v = 3u8; let s = S(&v); println!("{} {} {} {} {}", s.f(&4), s.g(), 5u8.h(&9), W("x").as_ref(), W("y") == "y"); }
