// Where clauses over lifetime parameters only (`where &'a A: Tr`)
// call the impl. Lust once kept them in the body env, so the calls
// went to the trait method's stub, an empty fn slot.
trait Tr {
    fn go(&self) -> i32;
    fn dflt(&self) -> i32 {
        100
    }
}

struct A;
impl<'x> Tr for &'x A {
    fn go(&self) -> i32 {
        8
    }
}

struct R<'a>(&'a i32);
impl<'a> Tr for R<'a> {
    fn go(&self) -> i32 {
        *self.0
    }
}

fn by_ref<'a>(x: &'a A) -> i32
where
    &'a A: Tr,
{
    x.go() + x.dflt()
}

fn by_struct<'a>(r: R<'a>) -> i32
where
    R<'a>: Tr,
{
    r.go() + r.dflt()
}

struct H<'a>(&'a A);
impl<'a> H<'a>
where
    &'a A: Tr,
{
    fn m(&self) -> i32 {
        self.0.go() + 1
    }
}

fn mixed<'a, T: Tr>(x: &'a A, t: T) -> i32
where
    &'a A: Tr,
{
    x.go() + t.go()
}

fn main() {
    let a = A;
    let five = 5;
    println!("{} {} {} {}", by_ref(&a), by_struct(R(&five)), H(&a).m(), mixed(&a, &A));
}
