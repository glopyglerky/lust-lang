// expect: L0001
// rustc: expected `]`, found `;`.
#[inline(never);]
fn f() {}

fn main() {
    f();
}
