// expect: L0612
// No `Into<String>` for `W`: the deferred obligation fails, as
// rustc's E0277 does.
struct W;
fn main() {
    let v: Vec<String> = vec![W].into_iter().map(Into::into).collect();
    println!("{v:?}");
}
