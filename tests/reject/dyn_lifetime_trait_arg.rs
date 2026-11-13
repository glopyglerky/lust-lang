// expect: L0203
// Valid Rust: lifetimes in a dyn type's trait args are not tracked yet.
trait Read<T> {
    fn get(&self) -> u8;
}

struct S;

impl<'a> Read<&'a i32> for S {
    fn get(&self) -> u8 {
        1
    }
}

fn read<'a>(r: &dyn Read<&'a i32>) -> u8 {
    r.get()
}

fn main() {
    println!("{}", read(&S));
}
