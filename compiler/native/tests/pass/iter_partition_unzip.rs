// Iterator::partition and unzip into pairs of Vecs as core 1.99: the fold's order, the
// predicate's calls, unzip's reserve of the size hint's lower bound, and drop order.
#[derive(Debug)] struct D(u32);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() {
    let (late, early): (Vec<usize>, Vec<usize>) = (0..7usize).partition(|&i| i % 3 == 0);
    println!("{late:?} {early:?} {} {}", late.capacity(), early.capacity());
    let (a, b): (Vec<D>, Vec<D>) = vec![D(1), D(2), D(3)].into_iter().partition(|d| { println!("test {}", d.0); d.0 != 2 });
    println!("{a:?} {b:?}");
    let (x, y): (Vec<u8>, Vec<String>) = (0..5u8).map(|i| (i, i.to_string())).unzip();
    println!("{x:?} {y:?} {} {}", x.capacity(), y.capacity());
    let (p, q): (Vec<u32>, Vec<char>) = vec![(1u32, 'a'), (2, 'b')].into_iter().filter(|_| true).unzip();
    println!("{p:?} {q:?} {} {}", p.capacity(), q.capacity());
    let (s, t): (Vec<D>, Vec<u8>) = vec![(D(7), 1u8)].into_iter().unzip();
    let (e1, e2): (Vec<u8>, Vec<u8>) = Vec::<(u8, u8)>::new().into_iter().unzip();
    println!("{s:?} {t:?} {} {}", e1.capacity(), e2.len());
    let v = vec![10u32, 20, 30];
    let (lo, hi): (Vec<&u32>, Vec<&u32>) = v.iter().partition(|&&x| x < 15);
    println!("{lo:?} {hi:?}");
}
