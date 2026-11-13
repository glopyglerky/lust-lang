// A raw pointer to str is a fat pointer (tf-fatptr; this was a lust
// reject before).
fn main() {
    let s = "hi";
    let p: *const str = s;
    println!("{}", !p.is_null());
}
