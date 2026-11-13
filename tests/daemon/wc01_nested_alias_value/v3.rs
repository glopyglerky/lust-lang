// A type alias inside a written type's argument changes under an unchanged body (no error,
// other output): the body must check and generate again (fn-body read sets, 2026-10-20).
mod m {
    pub type A = u8;
}

fn f() -> String {
    let x: Option<m::A> = Some(250);
    let y = x.map(|v| v.wrapping_add(10));
    format!("{:?} {}", y, std::mem::size_of::<Option<m::A>>())
}

fn main() {
    println!("{}", f());
}
