// A cast whose target names the fn's lifetime: `Self::slow as fn(&mut Self, TableKey<'a>, bool) -> u32`
// (ts-rust checker_p21.rs get_property_of_type_ex).
#[derive(Clone, Copy, Debug)]
pub enum TableKey<'a> { Text(&'a str), Id(u32) }
impl<'a> From<&'a str> for TableKey<'a> { fn from(s: &'a str) -> Self { TableKey::Text(s) } }
struct C { n: u32 }
fn opaque<F>(f: F) -> F { f }
impl C {
    fn slow(&mut self, k: TableKey<'_>, b: bool) -> u32 { self.n += 1; match k { TableKey::Text(s) => s.len() as u32 + b as u32, TableKey::Id(i) => i } }
    fn get_ex<'a>(&mut self, name: impl Into<TableKey<'a>>, b: bool) -> u32 {
        let name: TableKey<'a> = name.into();
        if self.n > 100 { return 0; }
        let lazy = opaque(Self::slow as fn(&mut Self, TableKey<'a>, bool) -> u32);
        lazy(self, name, b)
    }
}
fn main() {
    let mut c = C { n: 0 };
    let s = String::from("abcd");
    println!("{} {}", c.get_ex(s.as_str(), true), c.n);
}
