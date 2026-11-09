// expect: L0110
// rustc E0107: the crate-root `Option` shadows std's, and takes no args.
enum Option { Some(i32), None }
fn main() {
    let w: Option<i32> = None;
    println!("{}", w.is_none());
}
