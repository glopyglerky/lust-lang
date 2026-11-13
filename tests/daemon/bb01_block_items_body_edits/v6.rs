mod util { pub fn k() -> u32 { 1 } }
fn site(n: u32) -> u32 {
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Pt { x: u32, y: u32 }
    enum Mode { A, B(u32) }
    type Pair = (u32, u32);
    let p = Pt { x: n + 1, y: 2 };
    let m = if n > 3 { Mode::B(n) } else { Mode::A };
    let pr: Pair = (p.x, p.y);
    let v = match m { Mode::A => 0, Mode::B(b) => b };
    let q = p;
    let bad: u32 = "no";
    let s = format!("{:?}", q);
    p.x + pr.1 + v + s.len() as u32 + (p == q) as u32 + util::k()
}
fn main() { println!("{} {}", site(2), site(9)); }
