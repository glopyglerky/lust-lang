// expect: L0111
// rustc E0283: a global where clause does not pick between two impls
// (the default method made lust once infer i64 from it).
trait Conv<T> {
    fn conv(&self) -> T;
    fn dflt(&self) -> i32 {
        100
    }
}

struct A;
impl Conv<i64> for A {
    fn conv(&self) -> i64 {
        14
    }
}
impl Conv<u8> for A {
    fn conv(&self) -> u8 {
        15
    }
}

fn f()
where
    A: Conv<i64>,
{
    let x = A.conv();
    println!("{}", x);
}

fn main() {
    f();
}
