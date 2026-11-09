// Through a user Deref the first step is the target S: by-value methods on S
// win over &self methods (inherent or trait), as in rustc's probe order.
use std::ops::Deref;
use std::rc::Rc;
#[derive(Clone, Copy)]
struct S;
struct W(S);
impl Deref for W { type Target = S; fn deref(&self) -> &S { &self.0 } }
struct G<T>(T);
impl<T> Deref for G<T> { type Target = T; fn deref(&self) -> &T { &self.0 } }
trait Tr { fn m(self) -> &'static str; }
impl S { fn m(&self) -> &'static str { "inherent &self" } }
impl Tr for S { fn m(self) -> &'static str { "trait self" } }
#[derive(Clone, Copy)]
struct T2;
trait Tr2 { fn n(&self) -> &'static str; }
impl T2 { fn n(self) -> &'static str { "inherent self" } }
impl Tr2 for T2 { fn n(&self) -> &'static str { "trait &self" } }
fn main() {
    println!("{}", W(S).m());
    println!("{}", G(S).m());
    println!("{}", Box::new(W(S)).m());
    println!("{}", Rc::new(G(S)).m());
    println!("{}", G(T2).n());
    println!("{}", W(S).clone().m());
}
