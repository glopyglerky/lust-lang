// A user enum named `Option` shadows std's: its own methods, and
// std's Option still works through `Some`.
#[derive(Debug)]
enum Option<T> { Some(T), Nothing }
impl<T: std::fmt::Debug> Option<T> { fn show(&self) -> String { format!("{:?}", self) } }
fn main() {
    let a: Option<i32> = Option::Some(1);
    println!("{}", a.show());
    let b = Some(3u8).map(|x| x + 1);
    println!("{:?} {:?}", b, Option::<u8>::Nothing);
}
