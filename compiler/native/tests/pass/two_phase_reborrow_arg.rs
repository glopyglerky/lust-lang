// A `&mut` first argument reborrowed by coercion is two-phase in rustc:
// the other arguments may read the place (tsrust T10: regexp `simplify`).
struct N {
    sub: Vec<usize>,
}
fn simplify(nodes: &mut Vec<N>, re: usize) -> usize {
    if re == 0 {
        return 0;
    }
    let s = simplify(nodes, nodes[re].sub[0]);
    s + nodes.len()
}
fn push_len(v: &mut Vec<usize>, n: usize) {
    v.push(n);
}
fn main() {
    let mut v = vec![N { sub: vec![0] }, N { sub: vec![0] }];
    println!("{}", simplify(&mut v, 1));
    let mut w = vec![1];
    let x = &mut w;
    push_len(x, x.len());
    push_len(x, x[0]);
    println!("{:?}", w);
}
