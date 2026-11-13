fn check() -> String {
    #[derive(Clone, Default, Debug)]
    struct MemberInfo { missed: Vec<String>, name: String }
    let mut m: std::collections::BTreeMap<u32, MemberInfo> = std::collections::BTreeMap::new();
    m.insert(1, MemberInfo::default());
    format!("{:?}", m[&1])
}
fn main() { println!("{}", check()); }
