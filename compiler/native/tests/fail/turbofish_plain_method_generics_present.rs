// expect: L0721
// A turbofish on a method that is not generic, in a program that
// has generics (the generic method search used to accept it).
struct S;
impl S {
    fn get(&self) -> i64 {
        1
    }
}
fn id<T>(x: T) -> T {
    x
}
fn main() {
    println!("{} {}", S.get::<i64>(), id(2));
}
