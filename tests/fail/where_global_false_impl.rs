// expect: L0612
// rustc E0277: a global where clause on an impl block must hold.
trait Tr {
    fn go(&self) -> i32;
}

struct A;
struct B;

impl B
where
    A: Tr,
{
    fn m(&self) -> i32 {
        20
    }
}

fn main() {
    println!("{}", B.m());
}
