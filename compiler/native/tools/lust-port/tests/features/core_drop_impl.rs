// feature: core.drop_impl
// form: impl Drop with a side effect at scope end
struct Guard { name: &'static str }
impl Drop for Guard { fn drop(&mut self) { println!("drop {}", self.name); } }
fn main() { let _g = Guard { name: "a" }; println!("body"); }
