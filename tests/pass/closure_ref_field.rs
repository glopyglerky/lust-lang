// A field read through a shared reference captures the referent.
struct Cfg {
    scale: u32,
    name: String,
}
fn make(cfg: &Cfg) -> u32 {
    let f = |x: u32| x * cfg.scale;
    let g = || cfg.name.len() as u32;
    f(3) + g()
}
fn main() {
    let cfg = Cfg { scale: 3, name: String::from("abc") };
    println!("{}", make(&cfg));
    let r = &cfg;
    let h = move || r.scale + 1;
    println!("{} {}", h(), cfg.scale);
}
