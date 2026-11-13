// feature: attrs.cfg
// form: #[default] on an enum, rustfmt::skip, and test harness attributes in a normal build, as in ts-rust ast factory types
#[derive(Debug, Default, PartialEq, Eq)]
enum Mode {
    #[default]
    Fast,
    Slow,
}
#[rustfmt::skip]
fn pick() -> Mode { Mode::default() }
#[test]
#[ignore = "needs the full compiler"]
fn later() { panic!("not run"); }
#[test]
#[should_panic(expected = "boom")]
fn boom() { panic!("boom"); }
fn main() { println!("{:?} {}", pick(), pick() == Mode::Fast); }
