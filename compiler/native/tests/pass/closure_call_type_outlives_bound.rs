// A closure body calls a fn with a `T: 'a` bound with the enclosing fn's `T: 'a` (pop t_out_plain).
struct Q { n: u32 }
struct L<'a> { q: &'a Q }
trait Tr<'x> { fn get(&self) -> u32; }
impl<'x> Tr<'x> for &'x Q { fn get(&self) -> u32 { self.n } }
static SQ: Q = Q { n: 1 };
fn chain<'s, T: Copy + 's>(q: &'s Q, t: T) -> u8 {
    { let _t = t; } 0u8
}
fn user<'s, T: Copy + 's>(q: &'s Q, t: T) { let c = || { let inner = Q { n: 8 }; let g = chain(q, t); let _ = &g; }; c(); }
fn main() {}
