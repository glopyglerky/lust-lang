// capability: rejected: lifetimes in `dyn` trait arguments, `T: 'a` bounds on a type's parameters, and fns with `T: 'a` bounds used as values
// expect: L0203

trait T<'a> {
    fn f(&self) -> &'a u32;
}

fn g<'a>(x: &dyn T<'a>) -> &'a u32 {
    x.f()
}

fn main() {}
