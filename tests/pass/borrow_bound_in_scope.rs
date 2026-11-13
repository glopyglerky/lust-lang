// A trait named in a fn's bounds is in scope in its body (rustc's
// rule): `k.borrow()` under `K: Borrow<str>` needs no `use`, for a
// builtin key (`String`, through the conversion builtin) and for a
// user key with a written `Borrow` impl. The IndexMap lookups
// write exactly this.
fn get<K: core::borrow::Borrow<str>>(k: &K, key: &str) -> bool {
    key == k.borrow()
}

struct Key(String);
impl core::borrow::Borrow<str> for Key {
    fn borrow(&self) -> &str {
        &self.0
    }
}

fn main() {
    let s = String::from("alpha");
    let u = Key(String::from("beta"));
    println!("{} {} {} {}", get(&s, "alpha"), get(&s, "beta"), get(&u, "beta"), get(&u, "alpha"));
}
