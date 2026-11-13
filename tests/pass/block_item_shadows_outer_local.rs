// A block's fn item shadows an outer `let` of its name in the whole
// block, also before the item.
mod m { pub fn w() -> i32 { 3000 } }
fn main() {
    let w = m::w;
    {
        println!("{}", w());
        fn w() -> i32 { 253 }
    }
    println!("{}", w());
}
