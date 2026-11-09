// Statics: a Copy value loads, a non-Copy one is used in place (field
// reads, method calls, borrows), and a borrow is `'static`.
struct D(u8, &'static str);
impl D {
    fn name(&self) -> &'static str {
        self.1
    }
}
static N: u8 = 7;
static S: D = D(1, "s");
fn keep() -> &'static D {
    &S
}
fn main() {
    let n = N;
    let first = S.0;
    let r = &S;
    println!("{} {} {} {} {}", n, first, r.0, S.name(), keep().1);
}
