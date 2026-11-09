// Block items and uses under edits: a nested fn reads an outer block's
// item, a macro body reads the call site's block const, and a block
// `use` covers code before it.
fn g() -> i32 { 1 }
const V: i32 = 10;
macro_rules! getv { () => { V } }
mod m { pub fn w() -> i32 { 100 } }
fn w() -> i32 { 200 }
fn main() {
    {
        fn b() -> i32 { g() }
        println!("{}", b());
    }
    const V: i32 = 20;
    println!("{}", getv!());
    let x = w();
    use m::w;
    println!("{}", x);
}
