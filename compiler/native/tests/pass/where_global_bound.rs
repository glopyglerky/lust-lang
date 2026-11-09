// A where clause that names no type parameter (rustc's global
// predicate, like `where A: Tr`) must hold where it is written, and
// bodies still call the impl through it. Lust once kept it in the
// body env: the call selected it as param evidence and called the
// trait method's stub, an empty fn slot (garbage output).
trait Tr {
    fn go(&self) -> i32;
    fn dflt(&self) -> i32 {
        100
    }
}

trait Sub: Tr {
    fn sub(&self) -> i32 {
        3
    }
}

trait Mk {
    const C: i32;
    fn mk() -> Self;
    fn gm<U: Copy>(&self, u: U) -> U;
    fn dflt_mk(&self) -> i32 {
        0
    }
}

struct A;
impl Tr for A {
    fn go(&self) -> i32 {
        1
    }
}
impl Sub for A {}
impl Mk for A {
    const C: i32 = 9;
    fn mk() -> Self {
        A
    }
    fn gm<U: Copy>(&self, u: U) -> U {
        u
    }
}

struct W<T>(T);
impl<T: Tr> Tr for W<T> {
    fn go(&self) -> i32 {
        self.0.go() + 10
    }
}

fn needs() -> i32
where
    A: Tr,
{
    A.go()
}

fn paths() -> i32
where
    A: Tr,
{
    <A as Tr>::go(&A) + Tr::go(&A) + A::go(&A) + A.dflt()
}

fn through_closure() -> i32
where
    A: Tr,
{
    let c = || A.go() + 1;
    c()
}

fn supertrait() -> i32
where
    A: Sub,
{
    A.go() + A.sub() + A.dflt()
}

fn assoc_items() -> i32
where
    A: Mk,
{
    A::C + <A as Mk>::C + A::mk().gm(5) + A.gm(2u8) as i32
}

fn nested_impl() -> i32
where
    W<A>: Tr,
{
    W(A).go() + W(A).dflt()
}

struct B;
impl B {
    fn method(&self) -> i32
    where
        A: Tr,
    {
        A.go() + 10
    }
}

struct C;
impl C
where
    A: Tr,
{
    fn in_impl(&self) -> i32 {
        A.go() + 20
    }
}

trait T2 {
    fn req(&self) -> i32
    where
        A: Tr;
    fn with_default(&self) -> i32
    where
        A: Tr,
    {
        A.go() + 50
    }
}
impl T2 for B {
    fn req(&self) -> i32
    where
        A: Tr,
    {
        A.go() + 60
    }
}

// A template whose where clause also holds a global predicate.
fn generic<T: Tr>(t: T) -> i32
where
    A: Tr,
{
    A.go() * 1000 + t.go() + needs()
}

fn main() {
    println!("{} {} {}", needs(), paths(), through_closure());
    println!("{} {} {}", supertrait(), assoc_items(), nested_impl());
    println!("{} {} {} {}", B.method(), C.in_impl(), B.req(), B.with_default());
    println!("{} {}", generic(W(A)), generic(A));
}
