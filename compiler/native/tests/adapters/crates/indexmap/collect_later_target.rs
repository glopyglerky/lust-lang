// `collect` whose target a later use fixes: the closure result of
// Option::map, then unwrap_or_default into a field (ts-rust etscore.rs:808).
use indexmap::{IndexMap, IndexSet};

struct Opts {
    aliases: IndexMap<String, String>,
    names: IndexSet<u8>,
}

fn as_map(v: &IndexMap<String, String>) -> Option<&IndexMap<String, String>> {
    Some(v)
}

fn main() {
    let mut src = IndexMap::new();
    src.insert("b".to_string(), "2".to_string());
    src.insert("a".to_string(), "1".to_string());
    let mut m: IndexMap<String, IndexMap<String, String>> = IndexMap::new();
    m.insert("importAliases".to_string(), src);
    let o = Opts {
        aliases: m
            .get("importAliases")
            .and_then(as_map)
            .map(|a| a.iter().filter_map(|(k, v)| Some((k.clone(), v.parse::<u8>().ok()?.to_string()))).collect())
            .unwrap_or_default(),
        names: None.map(|v: &Vec<u8>| v.iter().copied().collect()).unwrap_or_default(),
    };
    println!("{:?} {:?}", o.aliases, o.names);
    let later = [3u8, 1, 3].iter().copied().collect();
    let s: IndexSet<u8> = later;
    println!("{:?}", s);
}
