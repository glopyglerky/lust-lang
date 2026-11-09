// feature: lifetimes.hrtb
// form: a `for<'a>` fn pointer alias whose output names the binder in Adt args, as in ts-rust ast/synthetic.rs `ListSel`
struct NodeData { v: Vec<u8> }
struct AnyList<'a> { s: &'a [u8] }
pub type ListSel = for<'a> fn(&'a NodeData) -> Option<Option<AnyList<'a>>>;
fn sel(n: &NodeData) -> Option<Option<AnyList<'_>>> { Some(Some(AnyList { s: &n.v })) }
fn count(f: ListSel, n: &NodeData) -> usize { match f(n) { Some(Some(l)) => l.s.len(), _ => 0 } }
fn main() { let n = NodeData { v: vec![1, 2] }; let t: [ListSel; 1] = [sel]; println!("{}", count(t[0], &n)); }
