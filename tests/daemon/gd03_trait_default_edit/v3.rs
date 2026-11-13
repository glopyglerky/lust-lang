trait Get<T = i32> {
    fn get(&self) -> T;
}
struct X;
impl Get for X {
    fn get(&self) -> i32 {
        7
    }
}
fn twice<A: Get>(a: &A) -> i32 {
    a.get() * 2
}
fn main() {
    println!("{}", twice(&X));
}
