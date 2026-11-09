// feature: traits.generics
// form: a blanket impl over every type with another trait, as in ts-rust helper traits
trait Named { fn name(&self) -> &str; }
trait Label { fn label(&self) -> String; }
impl<T: Named> Label for T { fn label(&self) -> String { format!("<{}>", self.name()) } }
struct Node;
impl Named for Node { fn name(&self) -> &str { "node" } }
fn main() { println!("{}", Node.label()); }
