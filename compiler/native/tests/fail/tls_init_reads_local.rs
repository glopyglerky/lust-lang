// expect: L0101
// The initializer is an item: it cannot see the enclosing fn's locals (E0435).
fn main() {
    let local = 5;
    thread_local! {
        static S: i32 = local;
    }
    S.with(|v| println!("{}", v));
}
