// feature: attrs.non_exhaustive
// form: non_exhaustive struct and enum, built and matched in the defining crate
#[non_exhaustive]
struct Opts { level: i32 }
#[non_exhaustive]
enum Mode { Fast, Slow }
fn main() {
    let o = Opts { level: 2 };
    let m = Mode::Slow;
    let n = match m { Mode::Fast => 1, _ => 2 };
    println!("{} {}", o.level, n);
}
