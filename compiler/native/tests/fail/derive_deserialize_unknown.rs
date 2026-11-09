// expect: L0403
// rustc error here: cannot find derive macro `Deserialize` in this
// scope (no serde in a single file). With serde linked, lust still
// rejects the derive itself: serde is a named gap (feature
// ext.serde), never a silent accept that drops the impl.
#[derive(Debug, Deserialize)]
struct Cfg {
    name: String,
}
fn main() {
    let c = Cfg { name: "a".to_string() };
    println!("{}", c.name);
}
