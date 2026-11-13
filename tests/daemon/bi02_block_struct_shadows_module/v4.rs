mod other { #[derive(Debug, Default)] pub struct MemberInfo { pub z: u8 } }
use other::MemberInfo;
fn check() -> String {
    let mut m: std::collections::BTreeMap<u32, MemberInfo> = std::collections::BTreeMap::new();
    m.insert(1, MemberInfo::default());
    {
        #[derive(Debug, Default)]
        struct MemberInfo { count: u64 }
        let mut i = MemberInfo::default();
        i.count += 2;
        println!("{:?}", i);
    }
    m.get_mut(&1).unwrap().z += 7;
    format!("{:?}", m[&1])
}
fn main() { println!("{}", check()); }
