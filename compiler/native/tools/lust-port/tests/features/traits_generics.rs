// feature: traits.generics
// form: user trait with a default method, an impl, and a generic fn with a bound
trait Named { fn name(&self) -> &str; fn label(&self) -> String { format!("[{}]", self.name()) } }
struct Node { kind: &'static str }
impl Named for Node { fn name(&self) -> &str { self.kind } }
fn show<T: Named>(x: &T) -> String { x.label() }
fn main() { println!("{}", show(&Node { kind: "ident" })); }
