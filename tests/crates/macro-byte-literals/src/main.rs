#[macro_use]
mod m;
mod user;

fn main() {
    let y = b"zz";
    let c = b'q';
    let a = textual_bytes!();
    let r = textual_raw!();
    let k = textual_byte!();
    let p = crate::path_bytes!();
    let q = crate::path_byte!();
    println!("{:?} {} {:?} {:?} {} {:?} {} {}", y, c, a, r, k, p, q, user::f());
}
