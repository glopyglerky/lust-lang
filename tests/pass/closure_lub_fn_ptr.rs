// Capture-free closures that differ join as a fn pointer in if, match,
// and array literals; one closure type repeated stays a closure array.
fn main() {
    let c = std::env::args().count();
    let f = match c { 0 => |x: u8| x + 1, 1 => |x: u8| x + 2, _ => |x: u8| x * 2 };
    let g = |x: u8| x;
    let same = [g, g];
    println!("{} {} {} {}", f(3), (same[0])(9), std::mem::size_of_val(&same), arrays());
}

fn arrays() -> u8 {
    let v = [|x: u8| x + 1, |x: u8| x * 3];
    (v[1])(5)
}
