// expect: L0612
// A `&str` searcher is not double-ended, so `matches` over one
// has no `rev` (tf-str).
fn main() {
    let mut it = "a.b".matches(".").rev();
    println!("{:?}", it.next());
}
