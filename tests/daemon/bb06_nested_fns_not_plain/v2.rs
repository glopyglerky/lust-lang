mod util { pub fn k() -> u32 { 1 } }
fn site(names: &[u32]) -> u32 {
    let wanted: Vec<u32> = names.iter().filter(|n| **n > 0).cloned().collect();
    struct Candidate { v: u32, depth: u32 }
    struct VisitState<'a> { wanted: &'a Vec<u32>, found: Vec<Candidate>, min: u32 }
    fn visit(v: u32, depth: u32, state: &mut VisitState<'_>) -> bool {
        if state.wanted.contains(&v) {
            state.found.push(Candidate { v, depth });
            if depth < state.min {
                state.min = depth;
            }
        }
        (0..v % 3).any(|c| v > 1 && visit(v / 3 + c / 2, depth + 1, state))
    }
    fn pick<T: Copy + Into<u64>>(t: T) -> u64 {
        t.into() * 2
    }
    fn apply<F: Fn(u32) -> u32>(f: F, v: u32) -> u32 {
        f(v) + 1
    }
    let mut state = VisitState { wanted: &wanted, found: Vec::new(), min: u32::MAX };
    let extra = 2;
    for v in 1..20 + extra {
        visit(v, 0, &mut state);
    }
    let best: u32 = state.found.iter().filter(|c| c.depth == state.min).map(|c| c.v).sum();
    best + pick(3u8) as u32 + pick(4u32) as u32 + apply(|x| x * 3, 2) + util::k()
}
fn main() { println!("{}", site(&[3, 5, 8, 0])); }
