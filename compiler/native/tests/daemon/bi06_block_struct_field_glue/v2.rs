fn check() -> String {
    #[derive(Clone, Default, Debug)]
    struct MemberInfo { missed: Vec<String>, name: String, count: u64 }
    let mut m: std::collections::BTreeMap<u32, MemberInfo> = std::collections::BTreeMap::new();
    m.insert(1, MemberInfo::default());
    m.get_mut(&1).unwrap().count += 7;
    format!("{:?}", m[&1])
}
fn main() { println!("{}", check()); }
