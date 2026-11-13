// expect: L0110
// rustc E0530: a `let` binding cannot shadow a static declared in an
// enclosing block (the block's items are in scope from its start).
#![allow(non_snake_case)]
fn main() {
    static X: u32 = 7;
    {
        let X = 1u32;
        println!("{}", X);
    }
    println!("{}", X);
}
