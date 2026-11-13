// expect: L0507
// A `use` through a derived block item is ambiguous with an outer item
// of its name (rustc E0659: a macro-expanded name in import resolution).
#[allow(dead_code)]
enum Item { Q }
fn main() {
    #[derive(Clone)]
    #[allow(dead_code)]
    enum Item { N, S }
    use Item::N;
    let _ = N;
}
