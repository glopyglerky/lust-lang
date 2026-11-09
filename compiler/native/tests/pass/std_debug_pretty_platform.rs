// The alternate Debug form of Permissions and Ancestors expands as
// in rustc (fz2-platform): both go through the debug builders.
fn main() {
    let p = std::fs::metadata("/").unwrap().permissions();
    println!("{:?}", p);
    println!("{:#?}", p);
    let a = std::path::Path::new("a/b").ancestors();
    println!("{:?}", a);
    println!("{:#?}", a);
    let mut b = std::path::Path::new("a").ancestors();
    b.next();
    b.next();
    println!("{:?}", b);
    println!("{:#?}", b);
}
