// W-BORROW3: explicit `for<'a>` binders that mean what elision means: on a
// fn pointer type alias (ts-rust ast/synthetic.rs `ListSel`), in a struct
// field and a table, on `Fn` bounds and where clauses, and on `dyn Fn`.
struct NodeData { v: Vec<u8> }
struct AnyList<'a> { s: &'a [u8] }
type ListSel = for<'a> fn(&'a NodeData) -> Option<Option<AnyList<'a>>>;
struct Site { sel: for<'a> fn(&'a NodeData) -> Option<Option<AnyList<'a>>> }
fn sel(n: &NodeData) -> Option<Option<AnyList<'_>>> { Some(Some(AnyList { s: &n.v })) }
fn none(_n: &NodeData) -> Option<Option<AnyList<'_>>> { None }
fn count(f: ListSel, n: &NodeData) -> usize {
    match f(n) { Some(Some(l)) => l.s.len(), _ => 0 }
}
fn apply<F: for<'a> Fn(&'a i32) -> i32>(f: F, x: &i32) -> i32 { f(x) }
fn first<F>(f: F, x: &[i32]) -> i32 where F: for<'a> Fn(&'a [i32]) -> &'a i32 { *f(x) }
fn by_dyn(f: &dyn for<'a> Fn(&'a str) -> &'a str, s: &str) -> usize { f(s).len() }
fn main() {
    let n = NodeData { v: vec![1, 2, 3] };
    let table: [ListSel; 2] = [sel, none];
    let site = Site { sel };
    for f in table { println!("{}", count(f, &n)); }
    println!("{}", count(site.sel, &n));
    let v = 5;
    println!("{} {}", apply(|n| *n * 2, &v), first(|s| &s[1], &[4, 5, 6]));
    println!("{}", by_dyn(&|s| &s[1..], "abcd"));
}
