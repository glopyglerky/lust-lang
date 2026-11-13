// A block fn in a non-root module shadows the module item of its name;
// edits add and remove the block item and the module item.
mod w {
    fn inner(y: i64) -> i64 { y + 7 }
    fn host(x: i64) -> i64 {
        fn inner(y: i64) -> i64 { y * 5 }
        inner(x)
    }
    pub fn run() { println!("{}", host(3)); }
}
fn main() { w::run(); }
