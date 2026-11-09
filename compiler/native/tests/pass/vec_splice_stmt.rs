// Vec::splice as a statement: drained items drop first, the hole fills, the tail moves by the exact hint; capacities as alloc (R4).
#[derive(Debug, Clone, Copy, PartialEq)]
struct N(u32);
struct D(u32);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() {
    let mut v = vec![1, 2, 3, 4, 5];
    v.splice(1..3, [7, 8, 9]); println!("{v:?} {}", v.capacity());
    v.splice(0..0, [0]); println!("{v:?} {}", v.capacity());
    v.splice(4.., vec![10, 11]); println!("{v:?} {}", v.capacity());
    v.splice(..2, []); println!("{v:?} {}", v.capacity());
    let items = [N(1), N(2), N(3)];
    let mut left: Vec<N> = vec![N(9); 4];
    let at = 2;
    left.splice(at..at, items.iter().copied()); println!("{left:?} {}", left.capacity());
    let mut out = b"abc".to_vec(); out.splice(0..0, [b'.', b'/']); println!("{:?} {}", out, out.capacity());
    let mut e: Vec<u8> = Vec::new(); e.splice(.., [1, 2, 3]); println!("{e:?} {}", e.capacity());
    let mut w = Vec::with_capacity(10); w.extend([1, 2, 3]); w.splice(1..2, [5, 6, 7, 8]); println!("{w:?} {}", w.capacity());
    let mut dv = vec![D(1), D(2), D(3), D(4)];
    let mut c = 0;
    dv.splice(1..3, [5, 6, 7].iter().map(|&x| { c += 1; println!("make {x}"); D(x) }));
    println!("{:?} {c}", dv.iter().map(|d| d.0).collect::<Vec<_>>());
    let src = vec![String::from("x"), String::from("y")];
    let mut s = vec![String::from("a")]; s.splice(..1, src.iter().cloned().rev()); println!("{s:?}");
    let mut z = vec![1, 2, 3]; z.splice(..z.len() - 1, [9]); println!("{z:?}");
}
