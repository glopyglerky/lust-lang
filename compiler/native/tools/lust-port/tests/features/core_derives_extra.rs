// feature: core.derives_extra
// form: derived Default, PartialOrd and Ord used in comparisons
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key { major: i32, minor: i32 }
fn main() {
    let a = Key::default();
    let b = Key { major: 1, minor: 5 };
    let c = Key { major: 2, minor: 0 };
    println!("{} {} {}", a.major, if b < c { 1 } else { 0 }, std::cmp::max(a.major, c.major));
}
