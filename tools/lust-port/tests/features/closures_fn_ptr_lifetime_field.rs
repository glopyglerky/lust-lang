// feature: closures.fn_ptr_lifetime_field
// form: a struct field whose fn pointer type names the struct's lifetime, fn(&'a str) -> &'a str
struct Hooks<'a> { first: fn(&'a str) -> &'a str }
fn head(s: &str) -> &str { &s[..1] }
fn main() {
    let h = Hooks { first: head };
    println!("{}", (h.first)("abc"));
}
