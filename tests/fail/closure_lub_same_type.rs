// expect: L0110
// rustc E0308: one closure type in both arms stays that closure type.
fn main() {
    let c = || 1;
    let v = if true { c } else { c };
    let _: fn() -> i32 = v;
    let _: () = v;
}
