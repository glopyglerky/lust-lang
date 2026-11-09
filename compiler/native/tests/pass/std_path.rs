// std::path (W-SHIM): components from both ends, parent, file name,
// stem, extension, join, push, pop, set_extension, ancestors, display,
// and component-wise comparison, on edge-case Unix paths.
use std::path::{Component, Path, PathBuf};
fn show(s: &str) {
    let p = Path::new(s);
    let mut comps: Vec<Component> = Vec::new();
    for c in p.components() { comps.push(c); }
    let mut back: Vec<Component> = Vec::new();
    let mut it = p.components();
    while let Some(c) = it.next_back() { back.push(c); }
    println!("{:?}: {:?} | {:?}", p, comps, back);
    println!("  parent={:?} name={:?} stem={:?} ext={:?} abs={} {:?}", p.parent(), p.file_name(), p.file_stem(), p.extension(), p.is_absolute(), p.components());
    println!("  disp=[{}] [{:>8}] {:?}", p.display(), p.display(), p.iter());
    let mut b = p.to_path_buf();
    println!("  set_ext={} {:?} with={:?} join={:?}", b.set_extension("rs"), b, p.with_extension(""), p.join("x/y"));
    let mut c = p.to_path_buf();
    println!("  pop={} {:?} fname={:?}", c.pop(), c, p.with_file_name("n.z"));
    let mut anc: Vec<&Path> = Vec::new();
    for a in p.ancestors() { anc.push(a); }
    println!("  anc={:?} sw={} ew={} strip={:?}", anc, p.starts_with("a"), p.ends_with("b.txt"), p.strip_prefix("/a"));
    println!("  prefix={:?} {:?}", p.file_prefix(), p.components().as_path());
}
fn main() {
    for s in ["", "/", "a", "a/b.txt", "/a/b.txt/", "./a/./b/..", "a//b/.", "..", ".hidden", "x.tar.gz", "/a/", ".", "./", "a/.", "../..", "foo.", "//x", ".a.b", "a/..b"] {
        show(s);
    }
    let a: PathBuf = "/a//b/".into();
    let c: PathBuf = "/a/c".into();
    println!("{} {} {}", a == *Path::new("/a/b"), a < c, Path::new("a") == "a/");
    println!("{:?} {:?} {}", Path::new("a/b").cmp(Path::new("a/./b/")), Path::new("a.b").cmp(Path::new("a/b")), *"./x" == *Path::new("x"));
    let mut b = PathBuf::new();
    b.push("x");
    b.push("/abs");
    b.push("y/");
    b.push("z");
    println!("{:?} {}", b, b.display());
}
