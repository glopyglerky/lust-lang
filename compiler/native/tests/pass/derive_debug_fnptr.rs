// Fn pointers implement Debug for every signature, including
// higher-ranked ones over `dyn FnMut` (W-BENCH3 bug 7).
#[derive(Debug)]
struct EmitHelper {
    name: &'static str,
    text_callback: Option<fn(make_unique_name: &mut dyn FnMut(&str) -> String) -> String>,
    plain: fn(u8) -> u8,
}
fn cb(make: &mut dyn FnMut(&str) -> String) -> String { make("x") }
fn inc(x: u8) -> u8 { x + 1 }
fn main() {
    let h = EmitHelper { name: "h", text_callback: Some(cb), plain: inc };
    let mut maker = |s: &str| format!("{s}!");
    println!("{} {} {}", h.name, h.text_callback.is_some(), (h.text_callback.unwrap())(&mut maker));
    println!("{}", (h.plain)(41));
    let n = EmitHelper { name: "n", text_callback: None, plain: inc };
    println!("{:?}", n.text_callback);
}
