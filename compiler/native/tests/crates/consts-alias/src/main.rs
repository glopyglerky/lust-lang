mod defs;

use defs::LIMIT;

fn main() {
    let x: defs::Num = LIMIT + defs::BASE;
    println!("consts {}", x);
}
