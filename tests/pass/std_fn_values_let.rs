// Std fns and the Option constructor as values, bound by `let` and
// called (`builtin_item_value`).
fn main() {
    let f = String::from;
    let s: String = f("x");
    let g = Box::new;
    let b = g(1u8);
    let h = Option::Some;
    println!("{} {} {:?}", s, b, h(2u8));
}
