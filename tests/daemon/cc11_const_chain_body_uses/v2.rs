const fn cf() -> i32 { "bad" }
const C: i32 = cf();
const D: i32 = C + 1;
const L: usize = D as usize;
fn kind(v: i32) -> &'static str {
    match v {
        C => "c",
        D => "d",
        _ => "other",
    }
}
fn main() {
    let x = const { D * 3 };
    let a = [1u8; L];
    println!("{} {} {} {}", x, a.len(), kind(2), kind(26));
}
