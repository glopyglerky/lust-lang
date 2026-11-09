// capability: attributes: path, macro_use, macro_export
// capability: items: mod (inline modules and `mod name;` files)
// expect: ok

#[path = "mods/helper.rs"]
mod helper;

fn main() {
    println!("{}", helper::value());
}
