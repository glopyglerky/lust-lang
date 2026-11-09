// expect: L0524
// rustc error: valid forms for the attribute are `ignore` and `ignore = "reason"`.
#[ignore()]
fn helper() {}
fn main() {}
