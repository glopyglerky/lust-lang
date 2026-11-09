// expect: L0305
// rustc E0509: `W<T>` implements Drop, and a `T` without a Copy bound
// moves, so the field cannot leave the value.
struct W<T>(T);

impl<T> Drop for W<T> {
    fn drop(&mut self) {
        println!("drop W");
    }
}

fn unwrap<T>(w: W<T>) -> T {
    w.0
}

fn main() {
    println!("{}", unwrap(W(5u8)));
}
