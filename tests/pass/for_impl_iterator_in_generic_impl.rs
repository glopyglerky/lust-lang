// for over an impl Iterator value in a generic impl (ts-rust effect/graph.rs:538 externals).
use std::collections::HashMap;
#[derive(PartialEq, Clone, Copy, Debug)] enum Dir { In, Out }
struct G<N> { nodes: Vec<(u32, N)>, adj: HashMap<u32, Vec<u32>> }
impl<N> G<N> {
    fn externals(&self, d: Dir) -> impl Iterator<Item = (u32, &N)> + '_ {
        self.nodes.iter().filter_map(move |(k, data)| { let a = if d == Dir::In { self.adj.get(k) } else { None }; if a.map_or(0, Vec::len) == 0 { Some((*k, data)) } else { None } })
    }
    fn starts(&self) -> Vec<u32> { let mut s = Vec::new(); for (idx, _) in self.externals(Dir::In) { s.push(idx); } s }
}
fn main() { let mut adj = HashMap::new(); adj.insert(2u32, vec![1u32]); let g = G { nodes: vec![(1, "a"), (2, "b")], adj }; println!("{:?} {:?}", g.starts(), Dir::Out); }
