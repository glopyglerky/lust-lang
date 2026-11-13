mod other { #[derive(Debug, Default)] pub struct MemberInfo { pub z: u8 } }
use other::MemberInfo as Outer;
fn check() -> String {
    #[derive(Clone, Default, Debug)]
    struct MemberInfo { missed: Vec<String>, name: String }
    let mut m: std::collections::BTreeMap<u32, MemberInfo> = std::collections::BTreeMap::new();
    m.insert(1, MemberInfo::default());
    m.get_mut(&1).unwrap().name.push('x');
    format!("{:?} {:?}", m[&1], Outer::default())
}
fn main() { println!("{}", check()); }
