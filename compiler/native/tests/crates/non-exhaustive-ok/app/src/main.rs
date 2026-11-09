fn main() {
    let o = base::opts();
    let base::Opts { level, .. } = o;
    let k = base::Kind::C { y: 5 };
    let kn = match k {
        base::Kind::A => 1,
        base::Kind::B(n) => n,
        base::Kind::C { y } => y,
        _ => 0,
    };
    let s = base::shape();
    let sn = match s {
        base::Shape::Rect { w, .. } => w,
        _ => 0,
    };
    println!("{} {} {} {}", level, kn, sn, base::opts().level);
}
