// expect: L0110
// rustc: type parameter `T` is only used recursively (E0392 class).
enum List<T> {
    Nil,
    Cons(Box<List<T>>),
}

fn main() {
    let _l: List<i32> = List::Nil;
}
