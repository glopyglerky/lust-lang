// expect: L0612
// rustc E0277: a global where clause must hold, even on a fn nobody calls.
trait Tr {
    fn go(&self) -> i32;
    fn dflt(&self) -> i32 {
        100
    }
}

struct A;

fn needs() -> i32
where
    A: Tr,
{
    5
}

fn main() {
    println!("hi");
}
