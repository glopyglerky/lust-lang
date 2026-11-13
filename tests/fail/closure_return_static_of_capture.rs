// expect: L0306
// rustc E0597: the closure returns a borrow of its capture as `&'static
// str`, so its creator needs `local` borrowed for `'static`.

fn main() {
    let local = String::from("l");
    let get = || -> &'static str { local.as_str() };
    println!("{}", get());
}
