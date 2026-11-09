mod util { pub fn k() -> u32 { 1 } }
fn site(n: u32) -> u32 {
    #[derive(Debug, Clone)]
    struct Pt { x: u8, y: u32 }
    impl Pt { fn sum(&self) -> u32 { self.x + self.y } }
    enum Mode { A, B(u32) }
    let p = Pt { x: n, y: 2 };
    let m = Mode::B(n);
    let v = match m { Mode::A => 0, Mode::B(b) => b };
    let s = format!("{:?}", p.clone());
    p.sum() + v + s.len() as u32 + util::k()
}
fn main() { println!("{}", site(5)); }
