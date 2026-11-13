// expect: L0110
// rustc E0392: a where clause is not a use of `T`.
struct S<T>
where
    T: Copy,
{
    x: i32,
}

fn main() {}
