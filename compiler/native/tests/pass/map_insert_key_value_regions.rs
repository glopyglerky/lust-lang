// HashMap<&'a str, &'static str>::insert(k, v): the key flows into the K slots
// and the value into the V slots, so a `'a` key needs no `'static` (ts-rust
// scanner number_cache, census "lifetime 'a must outlive 'static").
use std::collections::HashMap;
struct St<'a> { token_value: &'a str }
struct Sc<'a> { st: St<'a>, number_cache: HashMap<&'a str, &'static str> }
fn intern(s: &str) -> &'static str { if s == "12" { "12" } else { "other" } }
impl<'a> Sc<'a> {
    fn f(&mut self) {
        let tv = self.st.token_value;
        if let Some(&cached) = self.number_cache.get(tv) {
            self.st.token_value = cached;
        } else {
            let v = intern(tv);
            self.number_cache.insert(tv, v);
            self.st.token_value = v;
        }
    }
}
fn main() { let mut s = Sc { st: St { token_value: "12" }, number_cache: HashMap::new() }; s.f(); s.f(); println!("{}", s.st.token_value); }
