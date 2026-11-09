// macro_rules! bodies name items at the call site: a block's const or fn
// beats the crate-root item of the name (only locals are hygienic).
const V: i32 = 1;
fn w() -> i32 { 10 }
macro_rules! getv { () => { V + w() } }
fn main() {
    println!("{}", getv!());
    {
        const V: i32 = 2;
        fn w() -> i32 { 20 }
        println!("{}", getv!());
    }
}
