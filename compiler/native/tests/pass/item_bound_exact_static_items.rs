// ts-rust tsconfig_p1.rs get_spelling_suggestion: through `impl IntoIterator<Item
// = T>`, `T` holds the items' regions (`'static` here), not the `self` borrow.
use std::collections::HashMap;
pub struct Opt { name: &'static str }
static O1: Opt = Opt { name: "alpha" };
pub fn suggest<T: Clone + Default, S: AsRef<str>>(name: &str, candidates: impl IntoIterator<Item = T>, mut get_name: impl FnMut(&T) -> S, mut compare: impl FnMut(&T, &T) -> i32) -> T {
    let mut best = T::default();
    for c in candidates { if get_name(&c).as_ref().len() == name.len() && compare(&c, &best) >= 0 { best = c; } }
    best
}
pub struct NameMap(pub HashMap<String, &'static Opt>);
impl NameMap {
    pub fn get_spelling_suggestion(&self, name: &str) -> Option<&'static Opt> {
        suggest(name, self.0.values().map(|option| Some(*option)), |option: &Option<&'static Opt>| option.map_or("", |option| option.name),
            |a: &Option<&'static Opt>, b: &Option<&'static Opt>| { let a = a.map_or("", |o| o.name); let b = b.map_or("", |o| o.name); a.cmp(b) as i32 })
    }
}
fn main() { let mut m = HashMap::new(); m.insert("alpha".to_string(), &O1); let n = NameMap(m); println!("{:?}", n.get_spelling_suggestion("alphb").map(|o| o.name)); }
